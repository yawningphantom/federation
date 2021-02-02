export type WithProps<P extends Props, T> = T & P

export interface ErrDisplay {
  readonly code: string
  readonly message: string
}

interface Props {
  [other: string]: any
}

export type CtorWithProps<P extends Props, T> = {
  new(props?: P): T & P & ErrDisplay
  readonly code: string
}

export type GetMessage<P extends Props, T> = (this: WithProps<P, T>, props: P) => string

export interface Formatters {
  [code: string]: GetMessage<any, any>
}

type ErrorsOf<F extends Formatters, T> = {
  [code in keyof F]: CtorWithProps<Parameters<F[code]>[0], T>
}

interface AnyCtor<T> {
  new(...args: any[]): T
}

export function extending<B extends AnyCtor<any>>(Base: B) {
  return errors

  function errors<F extends Formatters>(errors: F): ErrorsOf<F, InstanceType<B>> {
    const created: any = {}
    for (const [code, message] of Object.entries(errors)) {
      const errClass = class extends Base {
        public static readonly code = code
        constructor(...args: any[]) {
          typeof args[0]?.message ===  'string'
            ? super(args[0]?.message)
            : super()
          Object.assign(this, args[0])
        }
        get message() {
          return message.apply(this, [this])
        }
        get code() { return code }
      }
      Object.defineProperty(errClass, 'name', { value: code + 'Error' })
      created[code] = errClass
    }
    return created as ErrorsOf<F, InstanceType<B>>
  }
}

// export function errors<E extends Formatters>(errors: E): ErrorsOf<E> {
//   const created: any = {}
//   for (const [code, message] of Object.entries(errors)) {
//     const errClass = class extends Error {
//       public static readonly code = code
//       constructor(props: Props) { super(); Object.assign(this, props) }
//       get message() {
//         return message.apply(this, [this])
//       }
//       get code() { return code }
//     }
//     Object.defineProperty(errClass, 'name', { value: code + 'Error' })
//     created[code] = errClass
//   }
//   return created as ErrorsOf<E>
// }

export const errors = Object.assign(extending(Error), {extending})
export default errors
