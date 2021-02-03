//! Spec url handling
//!
//! `Spec`s are parsed from URL strings and extract the spec's:
//!   - **identity**, which is the URL excluding the version specifier,
//!   - **name**, which is the second-to-last path segment of the URL,
//!     (typically the name of the bare directive exported by the spec), and
//!   - **version**, specified in the last URL path segment.
//!
//! # Example:
//! ```
//! use using::*;
//! assert_eq!(
//!   Spec::parse("https://spec.example.com/specA/v1.0")?,
//!   Spec::new("https://spec.example.com/specA", "specA", (1, 0))
//! );
//! Ok::<(), SpecParseError>(())
//! ```

import { ASTNode, DirectiveLocationEnum, DirectiveNode, DocumentNode } from 'graphql'
import { URL } from 'url'
import data, { Data } from './data'
import { asResultFn, isErr, ok, Result } from './err'
import { asString, AsString } from './is'
import { Deserialized, DeserializedShape, obj, ObjOf, ObjShape } from './metadata'
import { Version } from './version'

import ERROR from './err'

export const ErrNoPath = ERROR `NoPath` (
  ({ url }: { url: URL }) => `spec url does not have a path: ${url}`)

export const ErrNoName = ERROR `NoName` (
  ({ url }: { url: URL }) => `spec url does not specify a name: ${url}`)

export const ErrNoVersion = ERROR `NoVersion` (
  ({ url }: { url: URL }) => `spec url does not specify a version: ${url}`)

export class Spec {
  constructor(
    public readonly identity: string,
    public readonly name: string,
    public readonly version: Version
  ) {}

  /// Parse a spec URL or throw
  public static parse(input: string): Spec {
    const result = this.decode(input)
    if (isErr(result)) throw result.toError()
    return result.ok
  }

  /// Decode a spec URL
  public static decode(input: string): Result<Spec> {
    const result = parseUrl(input)
    if (isErr(result)) return result
    const url = result.ok

    const path = url.pathname.split('/')
    const verStr = path.pop()
    if (!verStr) return ErrNoVersion({ url })
    const version = Version.parse(verStr)
    const name = path[path.length - 1]
    if (!name) throw ErrNoName({ url })
    url.hash = ''
    url.search = ''
    url.password = ''
    url.username = ''
    url.pathname = path.join('/')
    return ok(new Spec(url.toString(), name, version))
  }

  toString() {
    return `${this.identity}/${this.version}`
  }

  input(...nameInput: AsString) {
    const name = asString(nameInput)
    return <S extends ObjShape, R extends Repetition>(shape: S, repeatable: R, ...on: DirectiveLocationEnum[]): ObjOf<S> & Specified<DeserializedShape<S>, R> => {
      const struct = obj(shape)
      const column = repeatable === 'on'
        ? data <Deserialized<typeof struct>, any> `${this.identity}#${name}`
        : data <Deserialized<typeof struct>[], any> `${this.identity}#${name} (repeatable)`
            .orElse(() => []) as any
      const index = data <Binding<Deserialized<typeof struct>>[], DocumentNode> `index of ${this.identity}#${name} in document`
        .orElse(() => []) as any
      return Object.assign(struct, {
        spec: this,
        name,
        column,
        index,
        repeatable: repeatable === 'repeatable on',
        on
      })
    }
  }
}

const parseUrl = asResultFn((url: string) => new URL(url))
export interface Binding<T> {
  data: T
  directive: DirectiveNode
  on: ASTNode
}

type Repetition = 'on' | 'repeatable on'

export interface Specified<T, R extends Repetition='on'> {
  readonly spec: Spec
  readonly name: string
  readonly repeatable: boolean
  readonly column: R extends 'on' ? Data<T, ASTNode> : Data<T[], ASTNode>
  readonly index: Data<Binding<T>[], DocumentNode>
  readonly on: DirectiveLocationEnum[]
}

export const spec = (...input: AsString) =>
  Spec.parse(asString(input))
