import Schema from "../schema";

describe("the Schema class", () => {
  it("parses a basic core schema, extracting the core version", () => {
    const example = Schema.parse`
          schema @core(using: "https://lib.apollo.dev/core/v0.1")
          { query: Query }
        `;

    expect(example.errors.length).toEqual(0);
  });

  it("extracts all spec references and exposes them as `.using`", () => {
    const example = Schema.parse`
          schema @core(using: "https://lib.apollo.dev/core/v0.1")
          { query: Query }
        `;

    expect(example.using).toMatchInlineSnapshot(`
        Array [
          Object {
            "as": null,
            "using": Spec {
              "identity": "https://lib.apollo.dev/core",
              "name": "core",
              "version": Version {
                "major": 0,
                "minor": 1,
              },
            },
          },
        ]
      `);
  });

  describe("Schema.ok", () => {
    it("throws if there are errors on the document", () => {
      const example = Schema.parse({
        src: "extra-schema.graphql",
        text: `
          schema @core(using: "https://lib.apollo.dev/core/v0.1")
          { query: Query }

          # error: extra schema
          schema { query: Query }
        `,
      });
      expect(() => example.ok()).toThrowErrorMatchingInlineSnapshot(`
        "[DocumentNotOk] extra-schema.graphql:1:0: one or more errors on document
          - [ExtraSchema] extra-schema.graphql:5:11: extra schema definition ignored"
      `);
    });
  });
});
