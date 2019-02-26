function get_index() {
  return new Promise(function (resolve, reject) {
    let req = new XMLHttpRequest();
    req.onreadystatechange = function() {
        if (this.readyState == 4 && this.status == 200) {
            let res = JSON.parse(this.responseText);
            console.log(res);
            resolve( res );
        } else if (this.readyState == 4) {
            reject(this);
        }
    };
    req.open("GET", "/", true);
    req.setRequestHeader("accept", "application/json");
    req.send();
  });
}

function get_entry(name) {
  return new Promise(function (resolve, reject) {
    let req = new XMLHttpRequest();
    req.onreadystatechange = function() {
        if (this.readyState == 4 && this.status == 200) {
            console.log(this.response.getElementsByTagName("main")[0]);
            resolve(this.response.getElementsByTagName("main")[0]);
        }else if (this.readyState == 4) {
            reject(this);
        }
    };
    req.onerror = function() {  reject(this); }
    req.responseType = "document";
    req.open("GET", "/entry/"+encodeURIComponent(name), true);
    req.send();
  });
}
