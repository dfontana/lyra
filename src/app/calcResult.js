export default function CalcResult({ result, error, expression }) {
  let body = []
  if (result) {
    body = [{ v: result }]
  } else if (!error.start) {
    body = [{ v: error.message }]
  } else {
    body = [
      { v: expression.slice(1, error.start), cx: '' },
      { v: expression.slice(error.start, error.end + 1), cx: 'calcError' },
      { v: expression.slice(error.end + 1), cx: '' },
    ]
  }
  return (
    <div className='calcResult'>
      {body.map((c, i) => (
        <span key={i} className={c.cx}>
          {c.v}
        </span>
      ))}
    </div>
  )
}
