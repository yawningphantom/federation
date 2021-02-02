type Template = Parameters<typeof String.raw>

function isTemplate(input: any): input is Template {
  const [template, ...subs] = input
  return template &&
    Array.isArray(template?.raw) &&
    template.raw.length === subs.length + 1
}

type ToString = Pick<Object, 'toString'>
type IntoString = string | ToString
type DeferFunc = () => string | DeferFunc
type Inline = IntoString | DeferFunc
type Input = Template | Inline[]

interface Print {
  (...fromTemplate: Template): Print
  (lines: Iterable<ToString>): Print
  toString(): String
  readonly lines: ReadonlyArray<string>
}

function print(...first: Input): Print {
  const lines: string[] = []
  const print = Object.defineProperties(appender(lines), {
    lines: {
      get() { return lines }
    },
    toString: {
      configurable: false, writable: false,
      value() {
        return lines.join('\n')
      }
    }
  })
  return print(...first)
}

const ROOT_SCOPE = Object.create(null)
let SCOPE = ROOT_SCOPE

function slot<T>(defaultValue: T, name?: string) {
  const sym = Symbol(name)
  ROOT_SCOPE[sym] = defaultValue

  return Object.assign(access, {
    clear: {
      configurable: false,
      writable: false,
      value: clear
    }
  })

  function access(set?: T) {
    if (set === void 0) return SCOPE[sym]
    SCOPE[sym] = set
  }

  function clear() {
    if (SCOPE === ROOT_SCOPE) return
    delete SCOPE[sym]
  }
}

function scope<F extends () => any>(func: F): ReturnType<F> {
  const prev = SCOPE
  try {
    SCOPE = Object.create(SCOPE)
    return func()
  } finally {
    SCOPE = prev
  }
}

function appender(lines: string[] = []) {
  return append

  function append(...input: Input) {
    if (isTemplate(input)) {
      push(String.raw.apply(null, input))
      return append
    }


    while (input.length) {
      const line = input.shift()
      if (typeof line === 'string')
        lines.push(line)
      else if (isIterable<string>(line))
        for (const l of line) push(l)
    }
    return append
  }

  function push(line: any) {
    if (line == null) return

    line =
      typeof line === 'string'
        ? line
        :
      typeof line.toString === 'function'
        ? line.toString()
        :
        null

    if (typeof line !== 'string') return
    lines.push(line)
  }
}

function isIterable<T>(iter: any): iter is Iterable<T> {
  return typeof iter[Symbol.iterator] === 'function'
}

console.log(
  print
    `hello`
    `world 1, 2, 3`
    .toString()
)
