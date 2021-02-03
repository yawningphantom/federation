import type { ASTNode, DirectiveLocationEnum, DirectiveNode, DocumentNode } from 'graphql'
import { name, Using } from './specs/core'
import { Spec, Specified } from './spec'
import { Deserialize } from './metadata'
import { default as ERROR, Err, isOk } from './err'
import { set } from './data'

export interface Layer {
  (doc: DocumentNode): (req: Using) => DirectiveVisitor | null
}

export interface DirectiveVisitor {
  (directive: DirectiveNode, on: ASTNode, onErr: (...err: Err[]) => void): void
}

export type Extract<T=any> = Specified<any> & Deserialize<any, ASTNode>

const ErrBadMetadata = ERROR `BadMetadata` (
  () => `could not read metadata`
)

const ErrBadForm = ERROR `BadMetadataForm` (
  (props: { name: string }) => `could not read form ${props.name}`
)

export default function layer(...md: Extract[]) {
  const byId = new Map<string, Extract[]>()
  for (const d of md) {
    const id = d.spec.identity
    if (!byId.has(id)) byId.set(id, [])
    byId.get(id)!.push(d)
  }
  return (doc: DocumentNode) => (req: Using): DirectiveVisitor | null => {
    const active = (byId.get(req.using.identity) ?? [])
      .filter(x => x.spec.version.satisfies(req.using.version))
    if (!active.length) return null

    const byName = new Map<string, Map<ASTNode["kind"], Extract[]>>()
    for (const item of active) {
      add(item,
        forName(`${name(req)}__${item.name}`),
        forName(`${name(req)}`))
    }

    return visit

    function visit(directive: DirectiveNode, on: ASTNode, onErr: (...err: Err[]) => void) {
      const byKind = byName.get(directive.name.value)
      if (!byKind) return
      const extractors = byKind.get(on.kind)
      if (!extractors) return
      let succeeded = false
      let errs = []
      for (const md of extractors) {
        const result = md.deserialize(directive)
        if (isOk(result)) {
          succeeded = true
          if (md.repeatable)
            md.column(on).push(result.ok)
          else
            set(on, md.column, result.ok)
          md.index(doc).push({
            data: result.ok,
            directive,
            on,
          })
          break
        }
        errs.push(ErrBadForm({ name: md.name, node: directive }, result))
      }
      if (!succeeded) onErr(ErrBadMetadata({ node: directive }, ...errs))
    }

    function forName(name: string): Map<ASTNode["kind"], Extract[]> {
      const existing = byName.get(name)
      if (existing) return existing
      const created = new Map
      byName.set(name, created)
      return created
    }

    function add(item: Extract, ...indexes: Map<ASTNode["kind"], Extract[]>[]) {
      for (const loc of item.on) {
        const kind = locationToKind[loc]!
        for (const byKind of indexes) {
          if (!byKind.has(kind)) byKind.set(kind, [])
          byKind.get(kind)!.push(item)
        }
      }
    }
  }
}

const locationToKind: { [loc in DirectiveLocationEnum]?: ASTNode["kind"] } = {
  // // Request Definitions
  // QUERY: 'QUERY';
  // MUTATION: 'MUTATION';
  // SUBSCRIPTION: 'SUBSCRIPTION';
  // FIELD: 'FIELD';
  // FRAGMENT_DEFINITION: 'FRAGMENT_DEFINITION';
  // FRAGMENT_SPREAD: 'FRAGMENT_SPREAD';
  // INLINE_FRAGMENT: 'INLINE_FRAGMENT';
  // VARIABLE_DEFINITION: 'VARIABLE_DEFINITION';

  // Type System Definitions
  SCHEMA: 'SchemaDefinition',
  SCALAR: 'ScalarTypeDefinition',
  OBJECT: 'ObjectTypeDefinition',
  FIELD_DEFINITION: 'FieldDefinition',
  ARGUMENT_DEFINITION: 'InputValueDefinition',
  INTERFACE: 'InterfaceTypeDefinition',
  UNION: 'UnionTypeDefinition',
  ENUM: 'EnumTypeDefinition',
  ENUM_VALUE: 'EnumValue',
  INPUT_OBJECT: 'InputObjectTypeDefinition',
  INPUT_FIELD_DEFINITION: 'InputValueDefinition',
};
