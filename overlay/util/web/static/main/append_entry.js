async function append_entry(item_parm) {
  let name;
  if(item_parm.name) {
    name = item_parm.name;
  }else {
    name = item_parm;
  }

  let old = document.getElementById(name);
  if(old) {
    old.scrollIntoView();
    return;
  }

  let sections_container = await get_entry(name);
  let main = document.getElementsByTagName('main')[0];

  for(let section of sections_container.children) {
      console.log(section);
      main.appendChild(section);
      section.scrollIntoView();
  }
}
