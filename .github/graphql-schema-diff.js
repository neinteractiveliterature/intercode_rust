// @ts-check

const { loadSchema } = require("@graphql-tools/load");
const { UrlLoader } = require("@graphql-tools/url-loader");
const { GraphQLFileLoader } = require("@graphql-tools/graphql-file-loader");
const { diff } = require("@graphql-inspector/core");
const { groupBy } = require("lodash");

const intercodeRubySchemaUrl =
  "https://raw.githubusercontent.com/neinteractiveliterature/intercode/main/schema.graphql";

/**
 * @param {string[]} items
 */
function makeChecklist(items) {
  return items.map((item) => `- [ ] ${item}`).join("\n");
}

/**
 * @param {import('@graphql-inspector/core').Change[]} changes
 */
function groupChangesByType(changes) {
  return groupBy(changes, (change) =>
    (change.path ?? "Unknown path").replace(/\.(.*)$/, "")
  );
}

/**
 * @param {ReturnType<typeof groupChangesByType>} groupedItems
 * @param {(change: import('@graphql-inspector/core').Change) => string} transform
 */
function makeGroupedChecklists(groupedItems, transform) {
  return Object.entries(groupedItems)
    .map(
      ([groupKey, items]) => `
## ${groupKey}

${makeChecklist(items.map(transform))}
`
    )
    .join("\n\n");
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
    .sort((a, b) => (a.path ?? "").localeCompare(b.path ?? ""))
    .map((change) => change.path ?? "Unknown path");

  const missingInputs = missingTypes.filter((path) =>
    (path ?? "").match(/Input$/)
  );
  const missingPayloads = missingTypes.filter((path) =>
    (path ?? "").match(/Payload$/)
  );
  const missingOther = missingTypes.filter(
    (path) => !missingInputs.includes(path) && !missingPayloads.includes(path)
  );

  const otherChanges = result.filter(
    (change) => change.type !== "TYPE_REMOVED"
  );

  const breakingChanges = groupChangesByType(
    otherChanges.filter((change) => change.criticality.level === "BREAKING")
  );
  const nonBreakingChanges = groupChangesByType(
    otherChanges.filter((change) => change.criticality.level !== "BREAKING")
  );

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

${makeGroupedChecklists(breakingChanges, (change) => change.message)}

<details>
  <summary>Non-breaking changes</summary>

  ${makeGroupedChecklists(nonBreakingChanges, (change) => change.message)}
</details>
  `;

  console.log(JSON.stringify(message));
}

main();
