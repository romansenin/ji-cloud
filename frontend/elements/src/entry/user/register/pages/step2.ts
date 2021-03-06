import { LitElement, html, css, customElement, property } from "lit-element";
import "@elements/core/dividers/spacer-fourty";

const STR_TITLE = "Sign Up - Step 2";

@customElement("page-register-step2")
export class _ extends LitElement {
  static get styles() {
    return [
      css`
        .wrapper {
          display: flex;
        }
        .inside-wrapper {
          width: 624px;
        }
        .side-image {
          width: 480px;
          min-width: 300;
          min-height: 100vh;
          background-color: #def4ff;
          background-image: url("https://i.ibb.co/g9N7MLy/shapes-1.png");
          background-repeat: no-repeat;
          background-attachment: inherit;
          background-position: center;
        }
        .content-wrapper {
          padding: 80px;
        }
        h1 {
          font-size: 32px;
          font-weight: 900;
          color: #5662a3;
        }
        ::slotted([slot="topleft"]),
        ::slotted([slot="topright"]),
        ::slotted([slot="bottomleft"]),
        ::slotted([slot="bottomright"]) {
          max-width: 296px;
          min-width: 296px;
          margin-bottom: 40px;
        }
        ::slotted([slot="topleft"]),
        ::slotted([slot="bottomleft"]) {
          margin-right: 32px;
        }
        ::slotted([slot="input"]) {
          margin-top: 20px;
        }
        ::slotted([slot="passwordreminder"]) {
          text-align: end;
        }
        ::slotted([slot="subtitle"]) {
          white-space: nowrap;
        }
        ::slotted([slot="submit"]) {
          margin-top: 40px;
        }
        ::slotted([slot="location"]) {
          margin-bottom: 40px;
        }
        ::slotted([slot="noaccount"]:last-child) {
          margin-left: 4px;
        }
        .noaccount {
          display: flex;
          align-items: center;
          margin-top: 16px;
        }
        .two-row {
          display: flex;
        }

        .spacer {
          height: 20px;
        }
        .hidden {
          display: none;
        }
        .password-wrapper {
          position: relative;
        }
        .password-wrapper div {
          position: absolute;
          top: 33%;
          right: -76px;
        }
      `,
    ];
  }

  render() {
    return html`
      <div class="wrapper">
        <div class="side-image"></div>
        <div class="content-wrapper">
          <h1>${STR_TITLE}</h1>
          <slot name="subtitle"></slot>
          <div class="inside-wrapper">
            <div class="two-row">
              <slot name="topleft"></slot>
              <slot name="topright"></slot>
            </div>
            <div class="two-row">
              <slot name="bottomleft"></slot>
              <slot name="bottomright"></slot>
            </div>

            <slot name="location"></slot>
            <spacer-fourty></spacer-fourty>
            <slot name="language"></slot>
            <div class="spacer"></div>
            <div class="password-wrapper">
              <slot name="checkbox"> </slot>

              <p></p>
              <slot name="submit"></slot>
            </div>
            <div class="noaccount">
              <slot name="noaccount"></slot>
            </div>
          </div>
        </div>
      </div>
    `;
  }
}
