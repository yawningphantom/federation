import { ValueNode, DirectiveNode, EnumValueNode, FloatValueNode, IntValueNode, ListValueNode, ObjectValueNode, StringValueNode, NullValueNode, BooleanValueNode, VariableNode, ASTNode, ObjectFieldNode, ArgumentNode, isValueNode, DocumentNode } from 'graphql'
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

export interface Ok<T> {
  is: 'ok'
  ok: T
  err?: never
}

export interface Err<E> {
  is: 'err'
  ok?: never
  err: E
}

export function err<E>(err: E) {
  return {
    is: 'err' as 'err',
    err
  }
}

export function ok<T>(ok: T) {
  return {
    is: 'ok' as 'ok',
    ok
  }
}

export type Result<T, E=any> = Ok<T> | Err<E>

const EXPECTED_VALUE = 'NonNull type received null'

export type Maybe<T> = T | null | undefined
export type Must<T> = Exclude<T, null | undefined>

const isNullNode = (n: any): n is NullValueNode => n.kind === 'NullValue'

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
          if (result.ok == null)
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

export const int = scalar(
  'IntValue',
  repr => {
    const decoded = +repr
    if (Number.isNaN(decoded)) return err('NaN')
    if (!Number.isSafeInteger(decoded)) return err('Int out of range')
    return ok(decoded)
  }
)

export const float = scalar(
  'FloatValue',
  repr => {
    const decoded = +repr
    if (Number.isNaN(decoded)) return err('NaN')
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
          return err(EXPECTED_VALUE)
        const underlying = deserialize(node)
        if (underlying.ok == null && !underlying.err)
          return err(EXPECTED_VALUE)
        return underlying
      }
    }
  })
}

export function list<T, V extends ValueNode>(type: Serde<T, V>) {
  return slot<T[], ListValueNode>(
    (values: T[] = []) => ({
      kind: 'ListValue' as 'ListValue',
      values: values.map(v => type.serialize(v)).filter(hasValue)
    }) as any,
    (node: Maybe<ListValueNode>) => {
      const results = ((node as ListValueNode)?.values ?? [])
        .map(v => type.deserialize(v as any))
      const errors = results.filter(r => !!r.err)
      if (errors.length) return err(errors)
      return ok(results.filter(r => !r.err).map(r => r.ok!))
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
        return err('Expected object or directive')
      const md = metadata(node)
      const results = Object.entries(shape)
        .map(([name, type]) => ({
          name,
          field: md[name],
          result: type.deserialize(md[name])
        }))
      const errors = results.filter(({ result }) => result.err)
        .map(({ name, field, result }) =>
          new FieldErr(name, field, result.err))
      if (errors.length) return err(errors)

      const shaped = Object.fromEntries(
        results.map(({ name, result }) => [name, result.ok])
      ) as any
      return ok(shaped)
    }
  )
}

abstract class NodeErr {
  abstract readonly code: string
  abstract readonly message: string

  toString() {
    return this.message
  }
}

class ObjErr {
  constructor(
    public readonly obj: ASTNode,
    public readonly cause: any,
  ) {}
}

class FieldErr {
  constructor(
    public readonly name: string,
    public readonly field: ASTNode,
    public readonly cause: any
  ) {}



  toString() {
    return `${mapSource(this.field.loc)}: could not deserialize ${this.name}: ${this.cause.message}`
  }
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
