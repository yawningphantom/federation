import { ASTNode } from 'graphql'
import { AsString, asString, Fn, FnPropsOf } from './is'
import sourceMap, { Source, SourceMap } from './source-map'

export interface Err {
  readonly is: 'err',
  readonly code: string
  readonly message: string
  readonly source?: Source
  readonly doc?: ASTNode
  readonly node?: ASTNode
  readonly causes: (Err | Error)[]
  toString(mapSource?: SourceMap): string
  toError(mapSource?: SourceMap): Error
}

export interface Ok<T> {
  is: 'ok'
  ok: T
}

export function ok<T>(ok: T) {
  return {
    is: 'ok' as 'ok',
    ok
  }
}

export type Result<T> = Ok<T> | Err
export type OkTypeOf<R extends Result<any>> = R extends Result<infer T> ? T : never

export function isErr(o: any): o is Err {
  return o?.is === 'err'
}

export function isOk<T>(o: any): o is Ok<T> {
  return o?.is === 'ok'
}

export function asResultFn<F extends (...args: any) => any>(fn: F) {
  return apply

  function apply(...args: Parameters<F>): Result<ReturnType<F>> {
    try {
      return {
        is: 'ok',
        ok: fn.apply(null, args)
      }
    } catch(error) {
      const err = Object.create(FROM_ERROR)
      err.causes = [error]
      return err
    }
  }
}


export function sift<T>(results: Result<T>[]): [Err[], T[]] {
  const okays: T[] = [], errors: Err[] = []
  for (const r of results) {
    if (isOk(r))
      okays.push(r.ok)
    else
      errors.push(r as Err)
  }
  return [errors, okays]
}


export default function err(...code: AsString) {
  const codeStr = asString(code)
  return createWithFormatter

  function createWithFormatter<F extends Fn<any, string>>(fmt: F): (input?: FnPropsOf<F> | Partial<Err>, ...causes: (Err | Error)[]) => FnPropsOf<F> & Err {
    const proto = Object.create(BASE, {
      code: {
        get() { return codeStr }
      },
      message: {
        get() {
          return fmt.apply(this, [this])
        }
      }
    })
    return (props, ...causes) => Object.assign(Object.create(proto), props, { causes })
  }
}

const BASE = { is: 'err', toString, causes: Object.freeze([]), toError }
const FROM_ERROR = Object.create(BASE, {
  message: {
    get() {
      return this.causes[0]?.message
    }
  },
  code: {
    get() {
      return this.causes[0]?.code ?? 'UnknownError'
    }
  },
})

function toString(this: Err, mapSource: SourceMap = sourceMap(this.source)) {
  let str = `[${this.code ?? 'UNKNOWN'}] ${mapSource(this.node?.loc)}: ${this.message}`
  for (const cause of this.causes) {
    str += '\n  - ' + cause.toString(mapSource).split('\n').join('\n    ')
  }
  return str
}

function toError(this: Err, mapSource: SourceMap = sourceMap(this.source)) {
  const error = new Error(this.toString(mapSource))
  Object.assign(error, this)
  return error
}
