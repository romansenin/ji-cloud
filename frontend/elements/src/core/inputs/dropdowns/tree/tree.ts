import { MEDIA_UI } from "@utils/path";
import { LitElement, html, css, customElement, property } from "lit-element";
import { noChange, nothing } from "lit-html";
export type Display = "collapsed" | "expanded";

@customElement("dropdown-tree")
export class _ extends LitElement {
  static get styles() {
    return [
      css`
    .main-wrapper{
        border-color:#e6f0ff;
        border-style:solid;
        border-width: 2px 2px 2px 8px;
        width:848px;
        min-height:48px;
        border-radius:12px;  
        
    }
    .bordergreen {
        border-left: solid 8px #6eca90;
    }
    .inside-wrapper{
        display:flex;
        align-items:center;
        padding-top:12px;
        
        
    }
    .text-wrapper{
        display:flex; 
        align-items:center;
    }
    ::slotted([slot=children]) {
        margin-top: 8px;
        margin-left:16px;
        
    }
    img {
        margin: 0 8px;
    }
    .open img{
        transform: rotate(90deg);
    }
    ul.closed {
        display: none;
    }
    ul{
        margin:0;
        padding:0;
    }
    p{
        line-height:0;
        margin:0;
    }
    .open .sidearrow{
        display:none;
    }
    .downarrow {
        display:none;
    }
    .open .downarrow {
        display:block;
    }
    ::slotted(*){
      display:flex;
      margin-left:40px;
    }
    `,
    ];
  }

  @property()
  label: string = "";

  @property({ type: Boolean })
  open: boolean = false;

  @property()
  display:Display = "expanded";

  render() {
    const { label, open,display } = this;

    const icon = display === "expanded" ? "Icon_CollapseAll_24.svg"
    : display === "collapsed" ? "Icon_ExpandAll_24.svg"
    : nothing;

    return html`
      <div class="main-wrapper ${open ? "bordergreen open" : ""}">
        <div class="inside-wrapper">
          <div class="text-wrapper">
            <img-ui class="sidearrow" path="icon-chevron-categories-24-px.svg" alt=""></img-ui>
            <img-ui class="downarrow" path="icon-chevron-categories-24-px-active.svg" alt=""></img-ui>

            <p>${label}</p>
            <img-ui path="${icon}"></img-ui>

          </div>
        </div>
        <ul class="${open ? "open" : "closed"}">
          <slot></slot>
        </ul>
      </div>

        
    `;
  }
}
