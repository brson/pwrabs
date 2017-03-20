function pwraps(config) {
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
        
    var config = stringBuf();
    config.set(JSON.stringify(json));
    // transfers the ownership of config
    var verifier = Module._pwrabs_create(config.ptr);
    
    var passbuf = stringBuf();
    return {
        verify: function (pass) {
            passbuf.set(pass);
            var res = pwrabs_verify(passbuf.ptr);
            if res != 0 {
                var err = utf8ToString(res);
                throw err;
            }
        }
    };
}
