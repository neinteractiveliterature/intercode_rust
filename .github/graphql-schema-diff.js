const { loadSchema } = require("@graphql-tools/load");
const { UrlLoader } = require("@graphql-tools/url-loader");
const { GraphQLFileLoader } = require("@graphql-tools/graphql-file-loader");
const { diff } = require("@graphql-inspector/core");

const intercodeRubySchemaUrl =
  "https://raw.githubusercontent.com/neinteractiveliterature/intercode/main/schema.graphql";

function makeChecklist(items) {
  return items.map((item) => `- [ ] ${item}`).join("\n");
}

async function main() {
  const intercodeRubySchema = await loadSchema(intercodeRubySchemaUrl, {
    loaders: [new UrlLoader()],
  });

  const intercodeRustSchema = await loadSchema("./schema.graphql", {
    loaders: [new GraphQLFileLoader()],
  });

  const result = await diff(intercodeRubySchema, intercodeRustSchema);

  const missingTypes = result
    .filter((change) => change.type === "TYPE_REMOVED")
    .sort((a, b) => a.path.localeCompare(b.path))
    .map((change) => change.path);

  const missingInputs = missingTypes.filter((path) => path.match(/Input$/));
  const missingPayloads = missingTypes.filter((path) => path.match(/Payload$/));
  const missingOther = missingTypes.filter(
    (path) => !missingInputs.includes(path) && !missingPayloads.includes(path)
  );

  const otherChanges = result
    .filter((change) => change.type !== "TYPE_REMOVED")
    .sort((a, b) => {
      if (
        a.criticality.level === "BREAKING" &&
        b.criticality.level != "BREAKING"
      ) {
        return -1;
      }

      if (
        b.criticality.level === "BREAKING" &&
        a.criticality.level != "BREAKING"
      ) {
        return 1;
      }

      const typeCompare = a.path.localeCompare(b.path);
      if (typeCompare !== 0) {
        return typeCompare;
      }

      return a.message.localeCompare(b.message);
    })
    .map((change) => `${change.criticality.level}: ${change.message}`);

  const message = `
# Missing types

${makeChecklist(missingOther)}

<details>
  <summary>Missing input types</summary>

  ${makeChecklist(missingInputs)}
</details>


<details>
  <summary>Missing payload types</summary>

  ${makeChecklist(missingPayloads)}
</details>

# Other changes

${makeChecklist(otherChanges)}
  `;

  console.log(JSON.stringify(message));
}

main();
