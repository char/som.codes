const { minify } = require("html-minifier");

const htmlparser2 = require("htmlparser2");
const domhandler = require("domhandler");
const domutils = require("domutils");
const cssSelect = require("css-select");
const renderDOM = require("dom-serializer").default;

const hljs = require("highlight.js");

/**
* @param {string} html 
* @returns {domhandler.Node[]}
*/
function parseDOM(html) {
  return new Promise((resolve, reject) => {
    const parser = new htmlparser2.Parser(new domhandler.DomHandler((error, dom) => {
      if (error) {
        reject(error);
      } else {
        resolve(dom);
      }
    }));

    parser.write(html)
    parser.end();
  })

}

async function handle(op, data) {
  if (op === "minify") {
    return minify(data, {
      collapseWhitespace: true,
      removeOptionalTags: true,
      collapseBooleanAttributes: true,
      removeAttributeQuotes: true,
      removeEmptyAttributes: true,
    });
  }

  if (op === "highlight") {
    const dom = await parseDOM(data);
    const codeBlocks = cssSelect.selectAll("pre > code", dom);
    for (const codeBlock of codeBlocks) {
      const className = domutils.getAttributeValue(codeBlock, "class");
      const contents = domutils.getText(codeBlock);

      let result = null;
      if (className != null) {
        if (className.startsWith("language-")) {
          result = hljs.highlight(contents, { language: className.substring("language-".length) });
        } else {
          continue
        }
      } else {
        result = hljs.highlightAuto(contents);
      }

      const highlighted = await parseDOM(result.value);

      domutils.replaceElement(codeBlock, new domhandler.Element("code", {
        "class": `language-${result.language}`,
      }, highlighted))
    }

    return renderDOM(dom);
  }

  return "?";
}

function main() {
  const readline = require("readline");
  const rl = readline.createInterface(process.stdin, process.stdout);
  rl.on("line", l => {
    const { seq, op, data } = JSON.parse(l);
    handle(op, data).then(response => {
      console.log(JSON.stringify({ seq, data: response }));
    });
  });
}

main();
