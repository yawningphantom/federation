
type Fn<I=any, O=any> = (input: I) => O
type FnInputOf<F extends Fn> = Parameters<F>[0]

class Flow<I, O> {
  static from<F extends Fn<any>>(fn: F): Flow<FnInputOf<F>, ReturnType<F>> {
    return new Flow(fn)
  }

  constructor(
    public readonly fn: Fn<any, O>,
    public readonly prev?: Flow<any, any>
  ) {}

  to<F extends Fn<any, any>>(fn: F): Flow<I & FnInputOf<F>, ReturnType<F>> {
    return new Flow(fn, this)
  }

  toFn(): Fn<I, O> {
    const {chain} = this
    return (input: I) => {
      for (const fn of chain) {
        const update = fn(input)
        if (update !== input) {
          Object.assign(input, update)
        }
      }
      return input as any as O
    }
  }

  get chain(): Fn[] {
    const chain = (this.prev?.chain ?? [])
    chain.push(this.fn)
    return chain
  }
}

const z = Flow.from(() => ({ code: 'hello' }))
  .to(x => ({ x: x.code }))
  .toFn()
