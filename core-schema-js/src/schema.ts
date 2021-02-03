import { DirectiveNode, parse as parseSchema, visit } from 'graphql'
import type { DocumentNode, SchemaDefinitionNode } from 'graphql'

import { asSource, AsSource, Source } from './source-map'

import { data, set } from './data'

import ERROR, { isErr, isOk, Ok } from './err'

import { core, Using } from './specs/core'
import { metadata } from './metadata'
import { Err } from './err'
import { Layer } from './layer'
import { Binding, Specified } from './spec'

const ErrNoSchemas = ERROR `NoSchemas` (() =>
  `no schema definition found`)

const ErrExtraSchema = ERROR `ExtraSchema` (() =>
  `extra schema definition ignored`)

const ErrNoCore = ERROR `NoCore` (() =>
  `@core(using: "${core}") directive required on schema definition`)

const ErrCoreSpecIdentity = ERROR `NoCoreSpecIdentity` ((props: { got: string }) =>
  `the first @core directive must reference "${core.identity}", got: "${props.got}"`)

const ErrBadUsingRequest = ERROR `BadUsingRequest` (() =>
  `@core(using:) invalid`)

const ErrDocumentNotOk = ERROR `DocumentNotOk` (() =>
  `one or more errors on document`)

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
 * Errors in this document
 */
const errors = data<Err[], DocumentNode> `Document errors`
  .orElse(() => [])

export class Schema {
  public static parse(...input: AsSource): Schema {
    return new Schema(asSource(input))
  }

  constructor(public readonly source: Source) { }

  get document() { return document(this.source) }
  get errors() { return errors(this.document) }
  get schema() { return theSchema(this.document) }

  attach(...layers: Layer[]): this {
    const onErr = addError(this.document)
    const visitors = this.using.flatMap(
      req =>
        layers.map(layer => layer(this.document)(req)!)
    ).filter(Boolean)
    visit(this.document, {
      Directive(node, _key, ancestors: any) {
        for (const v of visitors) {
          v(node, ancestors[ancestors.length - 1], onErr)
        }
      }
    })
    return this
  }

  find<S extends Specified<any>>(md: S): S extends Specified<infer T> ? Binding<T>[] : never {
    return md.index(this.document) as any
  }

  get using() { return using(this.document) }

  ok(): ValidSchema {
    // Bootstrap if we haven't already
    using(this.document)
    const err = errors(this.document)
    if (err.length)
      throw ErrDocumentNotOk({
        node: this.document,
        source: this.source
      }, ...err).toError()
    return this as ValidSchema
  }
}

export default Schema

export interface ValidSchema extends Schema {
  readonly schema: SchemaDefinitionNode
}

const addError = data <(...err: Err[]) => void, DocumentNode> `Report a document error`
  .orElse(doc => {
    const src = source(doc)
    const docErrors = errors(doc)
    return (...errs: Err[]) => {
      for (const err of errs) {
        ;(err as any).source = src
        docErrors.push(err)
      }
    }
  })

const theSchema =
  data <SchemaDefinitionNode | undefined, DocumentNode>
    `The schema definition node`
    .orElse(doc => {
      let schema: SchemaDefinitionNode | undefined = void 0
      const report = addError(doc)
      for (const def of doc.definitions) {
        if (def.kind === 'SchemaDefinition') {
          if (!schema) {
            schema = def
            continue
          }
          const error = ErrExtraSchema({ doc, node: def })
          report(error)
        }
      }
      if (!schema) {
        const error = ErrNoSchemas({ doc })
        report(error)
      }
      return schema
    })


const using =
  data <Using[], DocumentNode>
  `Specs in use by this schema`
  .orElse(doc => {
    // Perform bootstrapping on the schema
    const schema = theSchema(doc)
    if (!schema) return []

    // Try to deserialize every directive on the schema element as a
    // core.Using input.
    //
    // This uses the deserializer directly, not checking the name of the
    // directive. We need to do this during bootstrapping in order to discover
    // the name of @core within this document.
    const using = (schema.directives ?? [])
      .filter(d => 'using' in metadata(d))
      .map(d => ({
        result: Using.deserialize(d),
        directive: d
      }))

    // Core schemas MUST reference the core spec as the first @core directive
    // on their schema element.
    //
    // Find this directive. (Note that this scan is more permissive than the spec
    // requires, allowing the @core(using:) dire)
    const coreReq = using.find(d =>
      isOk(d.result) &&
      d.directive.name.value === (d.result.ok!.as ?? core.name)
    ) as {
      result: Ok<Using>,
      directive: DirectiveNode
    }

    const report = addError(doc)

    if (!coreReq) {
      report(ErrNoCore({ doc, node: schema }))
      return []
    }

    const {result: {ok: coreUse}, directive} = coreReq

    if (coreUse.using.identity !== core.identity) {
      report(ErrCoreSpecIdentity({ doc, node: directive ?? schema, got: coreUse.using.identity }))
      return []
    }

    const requests = using.filter(u => u.directive.name.value === directive.name.value)
    const bad = requests.filter(u => isErr(u.result))
    const good = requests.filter(u => isOk(u.result))
    report(
      ...bad.map(
        bad =>
          ErrBadUsingRequest({ doc, node: bad.directive }, bad.result as Err)
      )
    )
    return good.map(u => (u.result as Ok<Using>).ok)
  })
