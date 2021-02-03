import { ASTNode, DirectiveNode, parse as parseSchema, visit } from 'graphql'
import type { DocumentNode, SchemaDefinitionNode } from 'graphql'

import sourceMap, { asSource, AsSource, Source, SourceMap } from './source-map'

import { Sel, select, Selection } from './proc'
import { data, set } from './data'

import ERROR, { isErr, isOk, Ok, sift } from './err'

const ErrNoSchemas = ERROR `NoSchemas` (() =>
  `core schemas must contain a GraphQL schema definition`
)

const ErrExtraSchema = ERROR `ExtraSchema` (() =>
  `extra schema definition ignored`
)

const ErrNoCore = ERROR `NoCore` (() =>
  `the first @core(using:) directive must reference the core spec itself`
)

const ErrBadUsingRequest = ERROR `BadUsingRequest` (() =>
  `@core(using:) invalid`
)

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
const errors = data <Err[], ASTNode> `Errors on each node`
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
          const error = ErrExtraSchema({ doc, node: def })
          errors(def).push(error)
          errors(doc).push(error)
        }
      }
      if (!schema) {
        const error = ErrNoSchemas({ doc })
        errors(doc).push(error)
      }
      return schema
    })


import { core, Using } from './specs/core'
import { Maybe, metadata } from './metadata'
import { Err } from './err'

const using =
  data <Using[], DocumentNode>
  `Specs in use by this schema`
  .orElse(doc => {
    const schema = theSchema(doc)
    if (!schema) return []
    const using = (schema.directives ?? [])
      .filter(d => 'using' in metadata(d))
      .map(d => ({
        result: Using.deserialize(d),
        directive: d
      }))

    const coreReq = using.find(d => isOk(d.result)) as {
      result: Ok<Using>,
      directive: DirectiveNode
    }

    if (!coreReq) {
      errors(doc).push(ErrNoCore({ doc, node: schema }))
      return []
    }

    const {result: {ok: coreUse}, directive} = coreReq

    if (coreUse.using.identity !== core.identity ||
        (directive.name.value !== (coreUse.as ?? core.name))) {
      errors(doc).push(ErrNoCore({ doc, node: directive ?? schema }))
      return []
    }
    const requests = using.filter(u => u.directive.name.value === directive.name.value)
    const bad = requests.filter(u => isErr(u.result))
    const good = requests.filter(u => isOk(u.result))
    errors(doc).push(
      ...bad.map(bad =>
        ErrBadUsingRequest({ doc, node: bad.directive }, bad.result as Err))
    )
    return good.map(u => (u.result as Ok<Using>).ok)
  })

const example = Schema.parse({
  src: 'example.graphql',
  text:
    `
      schema
        @core(using: "https://lib.apollo.dev/core/v0.1")
        @core(using: "https://lib.apollo.dev/core/v0.1")
      {
        query: Query
      }

      type Query {
        value: Int
      }
    `
  })
  .ok()

console.log(using(example.document))
example.errors.forEach((e, i) => {
  console.log('error #', i)
  console.log(e.toString(formatLoc(source(example.document))))
})
// for (const e of example.errors) {
//   console.log('--- error', )
// }
// console.log(example.errors.map(x => x.toString()).join('\n'))
