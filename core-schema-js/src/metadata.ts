import { ValueNode, DirectiveNode, EnumValueNode, FloatValueNode, IntValueNode, ListValueNode, ObjectValueNode, StringValueNode, NullValueNode, BooleanValueNode, VariableNode, ASTNode, ObjectFieldNode, ArgumentNode, isValueNode, DocumentNode, Location } from 'graphql'
import data from './data'
import { errors } from './errors'
import { asString, AsString } from './is'
import sourceMap, { SourceMap } from './source-map'

export interface Metadata {
  [key: string]: ValueNode
}

export const metadata = data <Metadata, DirectiveNode | ObjectValueNode>
  `Key value mapping over arguments or fields`
  .orElse(target => {
    const args = target.kind === 'Directive' ? target.arguments : target.fields
    const meta: any = {}
    for (const arg of args ?? []) {
      meta[arg.name.value] = arg.value
    }
    return meta
  })

export function isMetadataTarget(o: any): o is DirectiveNode | ObjectValueNode {
  const kind = o?.kind
  return kind === 'Directive' || kind === 'ObjectValue'
}

export type Kind = ASTNode["kind"]
export type NodeForKind<K extends Kind> = ASTNode & { kind: K }

export type RawKind = (
  EnumValueNode
  | FloatValueNode
  | IntValueNode
  | NullValueNode
)["kind"]


export type ScalarKind = RawKind | StringValueNode["kind"]

export interface Serialize<T, N> {
  serialize(value: T): Maybe<N>
}

export interface Deserialize<T, N> {
  deserialize(node: Maybe<N>): Result<T>
}

export type Deserialized<S extends Deserialize<any, any> | Serialize<any, any>> =
  S extends Deserialize<infer T, any>
    ? T
    :
  S extends Serialize<infer T, any>
    ? T
    : never

export type Serialized<S extends Serialize<any, any> | Deserialize<any, any>> =
  S extends Serialize<any, infer N>
    ? N
    :
  S extends Deserialize<any, infer N>
    ? N
    : never

export type Serde<T, N extends ASTNode>
  = Serialize<T, N> & Deserialize<T, N>

export type Maybe<T> = T | null | undefined
export type Must<T> = Exclude<T, null | undefined>

const isNullNode = (n: any): n is NullValueNode => n.kind === 'NullValue'


import ERROR, { isErr, isOk, ok, Result, sift } from './err'

const EReadField = ERROR `ReadField` (
  (props: { name: string }) => `could not read field "${props.name}"`
)

const EReadObject = ERROR `ReadObject` (
  ({ }) => `could not read object`
)

const EBadReadNode = ERROR `BadReadNode` (
  (props: { expected: string, node: Maybe<ASTNode> }) =>
    `expected node of type ${props.expected}, got ${props.node?.kind}`
)


export class Slot<T, I extends ASTNode, O extends ValueNode>
  implements
    Serialize<T, O>,
    Deserialize<T, I>
   {
  constructor(
    public readonly serialize: Serde<T, O>["serialize"],
    public readonly deserialize: Serde<T, I>["deserialize"]
  ) {}

  default(defaultValue: Must<T>): Slot<Must<T>, I, Exclude<O, NullValueNode>> {
    const {deserialize} = this
    return Object.create(this, {
      defaultValue: { get() { return defaultValue } },
      deserialize: {
        value(node: Maybe<I>): Result<Must<T>> {
          const result = deserialize(node)
          if (!isErr(result) && result.ok == null)
            return ok(defaultValue)
          return result as Result<Must<T>>
        }
      }
    })
  }

  get maybe(): Slot<Maybe<T>, I | NullValueNode, O | NullValueNode> {
    return maybe(this) as any
  }

  get must(): Slot<Must<T>, Exclude<I, NullValueNode>, Exclude<O, NullValueNode>> {
    return must(this) as any
  }
}


export function slot<T, D extends ValueNode>(
  serialize: Serde<T, D>["serialize"],
  deserialize: Serde<T, D>["deserialize"],
): Slot<T, D, D> {
  return new Slot(serialize, deserialize)
}

export function scalar<T, K extends ScalarKind>(
  kind: K,
  decode: (repr: string) => Result<T>,
  encode: (value: T) => string = v => String(v)
) {
  return slot<Maybe<T>, ValueNode>(
    (value: Maybe<T>) => {
      if (!value) return NullValue
      return {
        kind,
        value: encode(value)
      } as any
    },
    (node: Maybe<ValueNode>) => {
      if (node?.kind === kind && hasValue(node))
        return decode(node.value)
      return ok(null)
    }
  )
}

const EReadNaN = ERROR `ReadNaN`
  ((props: { repr: string }) => `"${props.repr}" decoded to NaN`)
const EReadIntRange = ERROR `ReadIntRange`
  ((props: { repr: string }) => `"${props.repr}" out of range for integers`)

export const int = scalar(
  'IntValue',
  repr => {
    const decoded = +repr
    if (Number.isNaN(decoded)) return EReadNaN({ repr })
    if (!Number.isSafeInteger(decoded)) EReadIntRange({ repr })
    return ok(decoded)
  }
)

export const float = scalar(
  'FloatValue',
  repr => {
    const decoded = +repr
    if (Number.isNaN(decoded)) return EReadNaN({ repr })
    return ok(decoded)
  }
)

export const str = scalar(
  'StringValue',
  repr => ok(repr),
)

export function customScalar<T>(
  decode: (repr: string) => Result<T>,
  encode: (value: T) => string = v => String(v)
) {
  return scalar('StringValue', decode, encode)
}

function maybe<S extends Slot<any, any, any>>({serialize, deserialize}: S):
  Slot<Maybe<Deserialized<S>>, Serialized<S> | NullValueNode, Serialized<S> | NullValueNode>
{
  return slot(
    (val: Maybe<Deserialized<S>>) => {
      if (val == null) return NullValue
      return serialize(val)
    },
    (node: Serialized<S> | NullValueNode) => {
      if (!node || isNullNode(node)) return ok(null)
      return deserialize(node)
    }
  )
}

function must<S extends Slot<any, any, any>>(type: S):
  Slot<Must<Deserialized<S>>, Exclude<Serialized<S>, NullValueNode>, Exclude<Serialized<S>, NullValueNode>>
{
  const {deserialize} = type
  return Object.create(type, {
    deserialize: {
      value(node: Maybe<Serialized<S>>): Result<Must<Deserialized<S>>> {
        if (!node || isNullNode(node))
          return EBadReadNode({ node: node!, expected: '(non-null)' })
        const underlying = deserialize(node)
        if (!isErr(underlying) && underlying.ok == null)
          return EBadReadNode({ node: node!, expected: '(non-null)' })
        return underlying
      }
    }
  })
}

const EReadList = ERROR `ReadList` (() => `error reading list`)

export function list<T, V extends ValueNode>(type: Serde<T, V>) {
  return slot<T[], ListValueNode>(
    (values: T[] = []) => ({
      kind: 'ListValue' as 'ListValue',
      values: values.map(v => type.serialize(v))
    }) as any,
    (node: Maybe<ListValueNode>) => {
      const results = ((node as ListValueNode)?.values ?? [])
        .map(v => type.deserialize(v as any))
      const [errors, okays] = sift(results)
      if (errors.length) return EReadList({ node }, ...errors)
      return ok(okays)
    }
  )
}



export interface ObjShape {
  [key: string]: Serde<any, any>
}

type DeserializedShape<S extends ObjShape> = {
  [K in keyof S]: Deserialized<S[K]>
}

export function obj<S extends ObjShape>(shape: S):
  Slot<
    Maybe<DeserializedShape<S>>,
    ObjectValueNode | NullValueNode | DirectiveNode,
    ObjectValueNode | NullValueNode
  >
{
  return slot(
    (value: DeserializedShape<S>) => {
      if (!value) return NullValue
      return {
        kind: 'ObjectValue',
        fields: serializeFields(shape, value, 'ObjectField')
      }
    },
    (node: Maybe<ObjectValueNode | NullValueNode | DirectiveNode>) => {
      if (!isMetadataTarget(node))
        return EBadReadNode({ node, expected: 'ObjectValueNode | DirectiveNode' })
      const md = metadata(node)
      const results = Object.entries(shape)
        .map(([name, type]) => ({
          name,
          field: md[name],
          result: type.deserialize(md[name])
        }))
      const errors = []
      const entries = []
      for (const {name, field, result} of results) {
        if (isErr(result))
          errors.push(EReadField({
            name,
            node: field
          }, result))
        if (isOk(result))
          entries.push([name, result.ok])
      }
      if (errors.length) return EReadObject({ node }, ...errors)
      return ok(Object.fromEntries(entries))
    }
  )
}

function serializeFields<
  S extends ObjShape,
  K extends 'ObjectField' | 'Argument'
>(
  shape: S,
  value: DeserializedShape<S>,
  kind: K
): K extends 'ObjectField' ? ObjectFieldNode[] : ArgumentNode[] {
  return Object.entries(shape)
    .map(([name, type]) => ({
      kind,
      name: { kind: 'Name' as 'Name', value: name },
      value: type.serialize(value[name])
    })) as any
}

const NullValue = { kind: 'NullValue' as 'NullValue' }

const hasValue = (o: any): o is { value: string } =>
  typeof o?.value === 'string'
