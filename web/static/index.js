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
            console.log(this.response);
            resolve(this.response);
        }else if (this.readyState == 4) {
            reject(this);
        }
    };
    req.onerror = function() {  reject(this); }
    req.open("GET", "/entry/"+encodeURIComponent(name), true);
    req.send();
  });
}

async function build_menu() {
  let items = await get_index();

  let ul = document.createElement("ul");

  for (let item of items) {
    let elm = document.createElement("a");
    elm.innerText = item.name;
    elm.href = "#"+item.name;
    elm.onclick = function() {
      make_entry(item);
    };

    let list_item = document.createElement("li");
    list_item.appendChild(elm);
    ul.appendChild(list_item);
  }
  document.body.appendChild(ul);
}

async function make_entry(item) {
  let old = document.getElementById(item.name);
  if(old) {
    old.scrollIntoView();
    return;
  }

  let section = document.createElement("section");
  section.classList.add("entry");
  CONTAINER.appendChild(section);
  let content = get_entry(item.name);

  let edit_button = document.createElement("button");
  edit_button.innerText = "🖊";
  edit_button.onclick = function(){
    alert("TODO EDIT");
  };
  edit_button.classList.add("edit");
  section.appendChild(edit_button);

  let remove_button = document.createElement("button");
  remove_button.innerText = "❌";
  remove_button.onclick = function(){
    section.classList.add("deleted");
    let cs = getComputedStyle(section);
    section.style.maxHeight = cs.height;
    let wait_remove = parseFloat(cs.transitionDuration.slice(0,-1) || "0") * 1000;
    section.style.maxHeight = "0px";

    setTimeout(function () {
        CONTAINER.removeChild(section);
    }, wait_remove);
  };
  remove_button.classList.add("close");
  section.appendChild(remove_button);

  let h = document.createElement("h1");
  h.innerText = item.name;
  h.id = item.name;
  h.ondblclick = function(){
    section.classList.toggle("fullscreen");
  };
  h.classList.add("heading");
  section.appendChild(h);

  let ul = document.createElement("ul");
  ul.classList.add("tags");
  for (let tag of item.tags) {
    let li = document.createElement("li");
    li.innerText = tag.name;
    //TODO value
    ul.appendChild(li);
  }

  section.appendChild(ul);

  let pre = document.createElement("div");

  section.appendChild(pre);

  section.style.opacity = "0";

  pre.innerHTML = await content;
  section.scrollIntoView();

  let cs = getComputedStyle(section);
  let height = cs.height;
  let wait_remove = parseFloat(cs.transitionDuration.slice(0,-1) || "0") * 1000;
  section.style.maxHeight = "0px";
  section.style.opacity = null;
  setTimeout(function () {
      section.style.maxHeight = null;
  }, wait_remove);
  section.style.maxHeight = height;
}

var CONTAINER;

function main() {
  CONTAINER = document.body;
  build_menu();
}
