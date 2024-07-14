import {
    ClassicEditor,
    Essentials,
    Bold,
    Italic,
    Font,
    Paragraph,
    Link,
    List,
    BlockQuote,
    Heading
} from 'ckeditor5';
let editor;

ClassicEditor
    .create( document.querySelector( '#editor' ), {
        plugins: [ Essentials, Bold, Italic, Font, Paragraph, List, Heading, Link, BlockQuote ],
        toolbar: [
            // 'heading',
            // '|',
            'bold',
            'italic',
            'fontSize',
            'link',
            'bulletedList',
            'numberedList',
            'blockQuote'
            ],
    }
  )
  .then(newEditor => {
    editor = newEditor;
  })
    .catch(e => console.error(e));


const form = document.querySelector("#publishForm");
let content = document.querySelector("#content");

form.addEventListener("submit", () => {
  content.value = editor.getData();
})
