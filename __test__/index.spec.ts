import test from 'ava'
import fs from 'fs'
import path from 'path'
import { fileURLToPath } from 'url'
import { dirname } from 'path'

import { plus100, detectLists } from '../index'

const __filename = fileURLToPath(import.meta.url)
const __dirname = dirname(__filename)

test('sync function from native code', (t) => {
  const fixture = 42
  t.is(plus100(fixture), fixture + 100)
})

test('detectLists function', (t) => {
  const filePath = path.join(__dirname, './test.html')
  // </html>`
  const html = fs.readFileSync(filePath, 'utf8').toString()
  const results = detectLists(html)
  console.log('test: ', JSON.stringify(results, null, 2))

  t.pass()
})
