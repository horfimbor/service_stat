class Stat extends HTMLElement {


    constructor() {
        super();

        this.attachShadow({ mode: 'open' });
        this._nb = 0
        this._last_register = ""
        this._update().bind(this)
        this._render()
    }

    connectedCallback() {
        this._my_interval = setInterval(this._update.bind(this), 1000)
    }

    disconnectedCallback() {
        clearInterval(this._my_interval)
    }

    _update(){
        fetch("http://localhost:8001/data", {
            method : "GET",
            headers: {
               cache : "no-cache"
            },
        })
        .then(res => res.text())
        .then(response => {
            let d = response.split(':', 2);
            this._nb = d[0];
            this._last_register = d[1];
            this._render()
        })
        .catch(err => {
            console.log({service:"stat", status:"KO", error:err})
            alert("sorry, cannot stat")
        });
    }

     _render(){
        this.shadowRoot.innerHTML = `<ul>
           <li>nb inscrit : ${this._nb}</li>
           <li>last inscrit : ${this._last_register}</li>
       </ul>`
    }
}

customElements.define('hf-stat', Stat);