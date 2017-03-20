const worker_source = "async_worker.js";

var pwraps = function() {
    worker = new Worker(worker_source);
    var pending = null;
    
    const utf8Encoder = new TextEncoder();
    const utf8Decoder = new TextDecoder();
    
    worker.onmessage = function(e) {
        var msg = e.data;
        switch (msg.type) {
            case "json":
                var str = utf8Decoder.decode(msg.data);
                var response = str; //JSON.parse(str);
                console.log(response);
                pending.fulfill(response);
                pending = null;
        }
    };
    
    function send(data) {
        var str = JSON.stringify(data);
        worker.postMessage(utf8Encoder.encode(str));
    }
    
    return {
        verify: function (config) {
            if (pending) {
                pending.reject();
            }
            return new Promise(function(fulfill, reject) {
                send(config);
                pending = { fulfill: fulfill, reject: reject };
            });
        }
    };
}();

function test() {
    var config = {
        username: "user",
        email: "user@example.com",
        password: "foo bar"
    };
    pwraps.verify(config).then(console.log);
}

function update() {
    var config = {};
    for (var name of ["username", "email", "password"]) {
        config[name] = document.getElementById(name).value;
    }
    pwraps.verify(config).then(function(out) {
        document.getElementById("output").innerText = out;
    });
}
    
