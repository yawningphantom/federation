import { ASTNode, DirectiveNode, parse as parseSchema, visit } from 'graphql'
import type { DocumentNode, SchemaDefinitionNode } from 'graphql'

import sourceMap, { asSource, AsSource, Source, SourceMap } from './source-map'

import { Sel, select, Selection } from './proc'
import { data, set } from './data'

export class Schema extends Sel implements Selection<Schema> {
  public static parse(...input: AsSource): Schema {
    return new Schema(asSource(input))
  }

  constructor(public readonly source: Source) { super() }

  get document() { return document(this.source) }
  get errors() { return errors(this.document) }
  get schema() { return theSchema(this.document) }

  ok(): ValidSchema {
    const err = errors(this.document)
    if (err.length) throw err
    return this as ValidSchema
  }
}

export default Schema

export interface ValidSchema extends Schema {
  readonly schema: SchemaDefinitionNode
}

/**
 * Schema source
 */
const source = data <Source> `Document source`

/**
 * Document AST Node
 */
const document = data <DocumentNode, Source> `Document AST Node` .orElse(
  src => set(parseSchema(src.text), source, src)
)

/**
 * Document errors
 *
 * Errors are reported on both the document node and the node on which
 * the error occurred
 */
const errors = data <Error[], ASTNode> `Errors on each node`
  .orElse(() => [])

/**
 *
 */
// const allErrors = data <Error[], Traversal> `All errors on document`
//   .orElse(traversal => traversal.errors)


interface Traversal {
  schema?: SchemaDefinitionNode,
  errors: Error[]
}

/**
 * Format a location within a source
 */
const formatLoc = data <SourceMap, Source | undefined> `Document source map`
  .orElse(sourceMap as any)

const theSchema =
  data <SchemaDefinitionNode | undefined, DocumentNode>
    `The schema definition node`
    .orElse(doc => {
      let schema: SchemaDefinitionNode | undefined = void 0
      for (const def of doc.definitions) {
        if (def.kind === 'SchemaDefinition') {
          if (!schema) {
            schema = def
            continue
          }
          const error = new ExtraSchema(doc, def)
          errors(doc).push(error)
          errors(def).push(error)
        }
      }
      if (!schema) {
        const error = new NoSchemas(doc)
        errors(doc).push(error)
      }
      return schema
    })


import { core, Using } from './specs/core'
import { metadata } from './metadata'

const using =
  data <Using[], DocumentNode>
  `Specs in use by this schema`
  .orElse(doc => {
    const schema = theSchema(doc)
    if (!schema) return []
    const using = (schema.directives ?? [])
      .filter(d => 'using' in metadata(d))
      .map(directive => ({
        directive,
        result: Using.deserialize(directive)
      }))
    const fail = using
      .map(r => r.result.err).filter(Boolean)
    errors(doc).push(...fail)
    const coreUse = using.find(u => !!u.result.ok)
    if ((coreUse?.result.ok?.using.identity !== core.identity) ||
        (coreUse?.directive.name.value !== (coreUse?.result.ok?.as ?? core.name))) {
      errors(doc).push(new NoCore(doc, coreUse?.directive ?? schema))
      return []
    }
    const requests = using.filter(u => u.directive.name.value === coreUse.directive.name.value)
    const bad = requests.filter(u => !!u.result?.err)
    const good = requests.map(u => u.result.ok!).filter(Boolean)
    errors(doc).push(
      ...bad.map(bad => new InvalidRequest(doc, bad.directive, bad.result.err!))
    )
    return good
  })


abstract class SchemaError extends Error {
  public static readonly code: string = 'UnknownSchemaError'

  constructor(
    public readonly document: DocumentNode,
    public readonly node: ASTNode = document
  ) {
    super()
  }

  get code(): string {
    return (this.constructor as any).code
  }

  get location(): string | null {
    return formatLoc(source(this.document))(this.node.loc)
  }

  abstract get message(): string
}

export class NoSchemas extends SchemaError {
  public static readonly code = 'NoSchemas'
  public readonly message = `Core schemas must contain a GraphQL schema definition`
}

export class ExtraSchema extends SchemaError {
  public static readonly code = 'ExtraSchema'

  get message() {
    return `${this.location}: extra schema definition ignored`
  }
}

export class NoCore extends SchemaError {
  public static readonly code = 'NoCore'
  public readonly message = `${this.location}: the first @core(using:) directive must reference the core spec itself`
}

export class InvalidRequest extends SchemaError {
  public static readonly code = 'InvalidRequest'
  constructor(
    public readonly doc: DocumentNode,
    public readonly node: DirectiveNode,
    public readonly cause: any
  ) { super(doc, node) }

  get message() {
    console.log(this.cause)
    return `${this.location}: invalid request (${this.cause})`
  }
}

const example = Schema.parse `
  schema
    @core(using: "https://lib.apollo.dev/core/v0.1")
    @core(using: "https://lib.apollo.dev/corez/v1")
    @core(using: "https://lib.apollo.dev/core/v0.1")
  {
    query: Query
  }

  type Query {
    value: Int
  }
`
  .ok()

console.log(using(example.document))
console.log(example.errors.map(x => x.message).join('\n'))
