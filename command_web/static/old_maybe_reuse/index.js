var CONTAINER;

async function main() {
  CONTAINER = document.body;

  let trigger = document.getElementsByClassName('trigger');

  for(let elm of trigger) {
      elm.dispatchEvent(new Event('input'));
  }
}

async function render_search(render_on, term) {
  let result = document.getElementById(render_on);
  let idx = await get_index();

  result.innerHTML =  h`
    <ul>
    ${
      idx
        .filter((e) => (e.name.toLowerCase().search(term.toLowerCase()) != -1) )
        .map((e) => h`
            <li onclick="render_entry(this.innerText.trim())"> ${ e.name } </li>
        `)
    }
    </ul>
  `;
}

async function render_entry(name) {
    let cont = await get_entry(name);

    document.body.innerHTML += h`
    <section class="entry">
      <h1 class="heading">${name}</h1>
      <div>${cont}</div>
    </section>
    `;
}
