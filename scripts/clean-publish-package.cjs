const fs = require('fs')
const path = require('path')

const pkgPath = path.resolve(process.argv[2] || 'package.json')
const pkg = JSON.parse(fs.readFileSync(pkgPath, 'utf8'))

for (const field of ['dependencies', 'devDependencies']) {
  if (pkg[field]) {
    delete pkg[field]
    console.log(`Removed ${field} from ${path.relative(process.cwd(), pkgPath) || pkgPath}`)
  }
}

fs.writeFileSync(pkgPath, JSON.stringify(pkg, null, 2) + '\n')
