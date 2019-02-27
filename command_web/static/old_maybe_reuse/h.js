function h(strings, ...args ) {
    let res = "";

    function escape(str) {
        let ret = "";

        function pad(x) {
            let res = ""+x;
            while(res.length < 4) {
               res = "0"+res;
            }
            return res;
        }

        for(let x of str) {
            let small_alpha = (x > 'a' && x <= 'z');
            let big_alpha   = (x > 'A' && x <= 'Z');
            let numeric     = (x > '0' && x <= '9');
            let white       = (x.trim() == "");

            if(small_alpha || big_alpha || numeric || white) {
                ret += x;
            }else {
                ret += "&#"+pad(x.charCodeAt(0))+";";
            }
        };

        return  ret;
    }

    function append(arg) {
        let res = "";

        if(arg && arg.escaped) {
            res += arg;
        }else if (arg && typeof arg == "string") {
            res += escape(arg);
        }else if (arg && typeof arg == "object") {
            for (let item of arg) {
                res += append(item);
            }
        }

        return res;
    }

    for(var i = 0; i < strings.length; i++) {
      res += strings[i];
      res += append(args[i]);
    }

    let esc = new Object();

    esc.escaped = true;
    esc.toString = function() {
        return res;
    };

    return esc;
}
