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

import { URL } from 'url'
import { errors } from './errors'
import { asString, AsString } from './is'
import { Version } from './version'

export class Spec {
  constructor(
    public readonly identity: string,
    public readonly name: string,
    public readonly version: Version
  ) {}

  /// Parse a spec URL
  public static parse(input: string): Spec {
    const url = new URL(input)
    const path = url.pathname.split('/')
    const verStr = path.pop()
    if (!verStr) throw new ERR.NoVersion({ url })
    const version = Version.parse(verStr)
    const name = path[path.length - 1]
    if (!name) throw new ERR.NoName({ url })
    url.hash = ''
    url.search = ''
    url.password = ''
    url.username = ''
    url.pathname = path.join('/')
    return new this(url.toString(), name, version)
  }
}

export const spec = (...input: AsString) =>
  Spec.parse(asString(input))

export const ERR = errors({
  NoPath: ({ url }: { url: URL }) => `spec url does not have a path: ${url}`,
  NoVersion: ({ url }: { url: URL }) => `spec url does not specify a version: ${url}`,
  NoName: ({ url }: { url: URL }) => `spec url does not specify a name: ${url}`,
})
