const TURNSTILE_KEY = "1x00000000000000000000AA";

class Newsletter {
  constructor() {
    this.form = document.getElementById("newsletter-form");
    if(!this.form) {
      throw new Error("Newsletter form not found.");
    }

    this.submitButton = this.form.querySelector('button[type=submit]');
    if(!this.submitButton) {
      throw new Error("Newsletter submit button not found.");
    }

    this.subscriptionSuccess = document.getElementById("subscription-success");
    this.subscriptionError = document.getElementById("subscription-error");
  }

  setup() {
    this.form.addEventListener("submit", (event) => {
      event.preventDefault();
      const email = event.target.elements.email.value;
      const name = event.target.elements.name.value;
      const turnstileToken = event.target.elements["cf-turnstile-response"].value;
      this.send({ email, name, turnstileToken })
    })
  }

  disable() {
    this.submitButton.disabled = true;
  }

  enable() {
    this.submitButton.disabled = false;
  }

  send(payload) {
    this.setLoading(true);
    fetch("/api/newsletter", {
      method: "POST",
      body: JSON.stringify(payload),
      headers: {
        "content-type": "application/json",
      },
    }).then(response => {
      if(response.ok) {
        this.handleSendSuccess();
      } else {
        this.handleSendError(new Error(`API responded with status ${response.status}`));
      }
    }).catch((error) => {
      this.handleSendError(error);
    })
  }

  setLoading(loading) {
    if(loading) {
      this.submitButton.innerText = "Loading...";
    } else {
      this.submitButton.innerText = "Register"
    }
  }

  handleSendSuccess() {
    this.setLoading(false);
    this.subscriptionSuccess.classList.remove("hidden");
  }

  handleSendError(error) {
    this.setLoading(false);
    console.error(">>> Error sending subscription: ", error);
    this.subscriptionError.classList.remove("hidden");
  }
}

function setupTurnstile(newsletter) {
  turnstile.render("#turnstile-widget", {
    sitekey: TURNSTILE_KEY,
    callback: () => {
      newsletter.enable();
    },
    "error-callback": (error) => {
      newsletter.disable();
      console.error(">>> Turnstile error: ", error);
    }
  })
}

function main () {
  const newsletter = new Newsletter();
  newsletter.setup();

  setupTurnstile(newsletter);
}

main();
