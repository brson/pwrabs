let PORTS = new Array();
let PORTS_next = 1;
let HANDLES = new Array();
let HANDLES_next = 1;
let IMAGE_PORTS = new Array();
let PANICED = false;

const utf8Encoder = new TextEncoder();
const utf8Decoder = new TextDecoder();

// a usable fetch.
function read_from_url(url) {
    return fetch(url).then(function(r) {
        if (r.ok) {
            return r.arrayBuffer();
        } else {
            throw `failed to read form ${url}: ${r.status}`;
        }
    })
}

function utf8ToString(ptr, len) {
    return utf8Decoder.decode(Module.HEAPU8.subarray(ptr, ptr+len));
}

function stringToUtf8(string) {
    return utf8Encoder.encode(string);
}

function assert_type(x, type) {
    if (typeof(x) != type) {
        throw `wrong type: expected ${type}, found ${typeof(x)}.`;
    }
}

function assert_ptr(x) {
    assert_type(x, "number");
    if (parseInt(x) != x || x <= 0) {
        throw "not a pointer!";
    }
}

let OPEN_DIRECTORIES = new Array();
function open_directory(name) {
    assert_type(name, "string");
    let handle = OPEN_DIRECTORIES[name];
    if (handle == undefined) {
        handle = add_handle(Directory(name));
    }
    return Promise.resolve(handle);
}

function Directory(base) {
    return {
        get_file: function(name) {
            assert_type(name, "string");
            return Promise.resolve(add_file(`${base}/${name}`));
        },
        get_directory: function(name) {
            assert_type(name, "string");
            let handle = add_handle(Directory(`${base}/${name}`));
            return Promise.resolve(handle);
        }
    };
}
function add_file(name) {
    return add_handle({
        read: function() {
            return read_from_url(name);
        }
    });
}

function mount(webfs) {
    return Directory(webfs.root);
}

function dirname(p) {
    let i = p.lastIndexOf('/');
    return p.substring(0, i);
}

function search(handles, name) {
    let promises = [];
    for (let h of handles) {
        promises.push(mountpoints[h].read(name));
    }
    return Promise.race(promises);
}

function read_file(handle) {
    return new Promise(function (resolve, reject) {
        let reader = new FileReader();
        reader.onload = function() { resolve(reader.result) };
        reader.onerror = function(e) { reject(e.type) };
        reader.readAsArrayBuffer(HANDLES[handle]);
    });
}

function add_handle(b) {
    let h = HANDLES_next;
    HANDLES_next += 1;
    HANDLES[h] = b;
    return h;
}
function add_url(url) {
    return add_handle({
        read: function() {
            return read_from_url(url);
        }
    });
}
const LOG_NAMES = {
    1:  "Trace",
    2:  "Debug",
    3:  "Info",
    4:  "Warn",
    5:  "Error"
};
function log(id, level, msg) {
    //let f = log_functions[level];
    //f(id, msg);
    console.log(id, msg);
}
function log_branch(parent, child) {}

function DataTask(task, is_stream) {
    assert_ptr(task);
    let _task = task;
    return {
        complete: function(data) {
            if (PANICED) return;
            let task = _task;
            if (!is_stream) {
                _task = null;
                if (task === null) return;
            }
            let arr = new Uint8Array(data);
            try {
                var buf_ptr = Module._task_complete_data(task, arr.byteLength);
                if (buf_ptr !== 0) {
                    Module.HEAPU8.set(arr, buf_ptr);
                }
                Module._task_resume_data(task);
            } catch(e) {
                PANICED = true;
                throw e;
            }
        },
    
        failed: function(msg) {
            assert_type(msg, "string");
            
            if (PANICED) return;
            let task = _task;
            if (!is_stream) {
                _task = null;
                if (task === null) return;
            }
            
            var msg_utf8 = stringToUtf8(msg);
            var utf8_len = msg_utf8.length;
            try {
                var buf_ptr = Module._task_failed_data(task, msg_utf8.length);
                if (buf_ptr !== 0) {
                    Module.HEAPU8.set(msg_utf8, buf_ptr);
                }
                Module._task_resume_data(task);
            } catch(e) {
                PANICED = true;
                throw e;
            }
        }
    };
};
function HandleTask(task, is_stream) {
    assert_ptr(task);
    let _task = task;
    return {
        complete: function(handle) {
            if (PANICED) return;
            let task = _task;
            if (!is_stream) {
                _task = null;
                if (task === null) return;
            }
            try {
                Module._task_resume_complete_handle(task, handle);
            } catch(e) {
                PANICED = true;
                throw e;
            }
        },
    
        failed: function(msg) {
            if (PANICED) return;
            let task = _task;
            if (!is_stream) {
                _task = null;
                if (task === null) return;
            }
            
            var msg_utf8 = stringToUtf8(msg);
            var utf8_len = msg_utf8.length;
            try {
                var buf_ptr = Module._task_failed_handle(task, msg_utf8.length);
                if (buf_ptr !== 0) {
                    Module.HEAPU8.set(msg_utf8, buf_ptr);
                }
                Module._task_resume_failed_handle(task);
            } catch(e) {
                PANICED = true;
                throw e;
            }
        }
    };
};

function port_register(p) {
    PORTS[++PORTS_next] = p
    return PORTS_next;
}

function channel_port(c) {
    let port = {
        send: function(data) {
            c.postMessage({
                type:   "json",
                data:   data
            });
        },
        send_image: function(id, image) {
            c.postMessage({
                type:   "image",
                data:   image,
                id:     id
            });
        },
        recv: null
    };
    c.onmessage = function(e) {
        port.recv(e.data);
    };
    return port_register(port);
}

var Module = {
    preInit: function() {
        let abort = Module["abort"];
        Module["abort"] = function() {
            PANICED = true;
            abort();
        };
    }
};
