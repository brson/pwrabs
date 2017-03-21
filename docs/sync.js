var pwraps = function() {
    const utf8Encoder = new TextEncoder();
    
    function stringToUtf8(string) {
        return utf8Encoder.encode(string);
    }
    
    function stringBuf() {
        var buf_ptr = Module._buf_create()
        return {
            set: function(str) {
                var data = stringToUtf8(str);
                var data_ptr = Module._buf_write(buf_ptr, data.byteLength);
                if (buf_ptr !== 0) {
                    Module.HEAPU8.set(data, data_ptr);
                }
            },
            ptr: buf_ptr
        };
    }
        
    var verifier = Module._pwrabs_create();
    
    var passbuf = stringBuf();
    return {
        verify: function (set) {
            passbuf.set(JSON.stringify(set));
            var res = Module._pwrabs_verify(passbuf.ptr);
            if (res != 0) {
                var err = JSON.parse(utf8ToString(res));
                throw err;
            }
        }
    };
}();

function test() {
    var config = {
        username: "user",
        email: "user@example.com",
        password: "foo bar"
    };
    pwraps.verify(config);
}
