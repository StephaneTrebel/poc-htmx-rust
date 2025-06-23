function getValue(element) {
  return element.value;
}

// Many thanks to https://dev.to/stuffbreaker/custom-forms-with-web-components-and-elementinternals-4jaj
try {
  customElements.define(
    'my-input',
    class extends HTMLElement {
      static formAssociated = true;
      static get observedAttributes() {
        return ['label', 'name', 'placeholder', 'required', 'type', 'value'];
      }

      constructor() {
        super();
        this._attrs = {};
        this._internals = this.attachInternals();
        this._internals.role = 'textbox';
        this.tabindex = 0;
      }

      connectedCallback() {
        this.innerHTML = `
			<div class="row g-3 align-items-center">
        <div class="col-auto">
          <label
            for="${this._attrs['name']}"
            class="form-label"
          >${this._attrs['label']}</label>
        </div>
        <div class="col-auto">
          <input
            id="${this._attrs['name']}"
            class="form-control"
            name="${this._attrs['name']}"
            type="${this._attrs['type'] || 'text'}"
            placeholder="${this._attrs['placeholder']}"
            hx-trigger="keyup changed delay:500ms"
            hx-get="/check-input"
            hx-target='#span-${this._attrs['name']}'
            hx-swap="innerHTML"
            hx-vals='js:{content: getValue(${this._attrs["name"]})}'
          />
        </div>
        <div class="col-auto" id="span-${this._attrs['name']}">
        </div>
      </div>
`;
        this.$input = this.querySelector('input');
        this.setProps();

        // Clear up validation state when inputting stuff
        // The first one is for the initial state
        this.$input.addEventListener('input', () => this.handleInput());
      }

      handleInput() {
        this._internals.setValidity(
          this.$input.validity,
          this.$input.validationMessage,
          this.$input
        );
      }

      attributeChangedCallback(name, _prev, next) {
        this._attrs[name] = next;
        this.setProps();
      }

      setProps() {
        // prevent any errors in case the input isn't set
        if (!this.$input) {
          return;
        }

        // loop over the properties and apply them to the input
        for (let prop in this._attrs) {
          switch (prop) {
            case 'value':
              this.$input.value = this._attrs[prop];
              break;
            case 'required':
              const required = this._attrs[prop];
              this.$input.toggleAttribute(
                'required',
                required === 'true' || required === ''
              );
              break;
          }
        }

        // reset the attributes to prevent unwanted changes later
        this._attrs = {};
      }
    }
  );
} catch (e) {
	// Prevent console pollution if the component already exists
	if (!e.toString().includes("already been defined as a custom element")) {
		throw e;
	}
}
