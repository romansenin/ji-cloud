import "@elements/column-details";
import "@elements/column-list";
 

export default {
    title: 'Homepage Paragraph',
  }


  const STR_TITLE="Help"
  const STR_LINE1="Support & FAQ"
  const STR_LINE2="Quick tour"
  const STR_LINE3="JI Tutorials"
  const STR_LINE4="Online webinars"
  const STR_LINE5="accessibility"
   
  




export const footerhelp= () => {

    return `
    <column-details head_title="${STR_TITLE}">
    <column-list text_line="${STR_LINE1}" ></column-list>
    <column-list text_line="${STR_LINE2}" ></column-list>
    <column-list text_line="${STR_LINE3}" ></column-list>
    <column-list text_line="${STR_LINE4}" ></column-list>
    <column-list text_line="${STR_LINE5}" ></column-list>
      </column-details>
   
    
    `
}










