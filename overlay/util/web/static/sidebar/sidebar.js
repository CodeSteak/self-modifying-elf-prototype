async function render_sidebar_search(target_id, term) {
  let target = document.getElementById(target_id);
  let idx = await get_index();
  let results = idx.filter((e) => (e.name.toLowerCase().search(term.toLowerCase()) != -1) );

  target.innerHTML = "";

  for(let item of results) {
    let a = document.createElement("a");
    a.innerText = item.name;
    a.href = "#"+item.name;
    a.onclick = function() {
        append_entry(item);
    };

    let list_item = document.createElement("li");
    list_item.appendChild(a);
    target.appendChild(list_item);
  }
}
