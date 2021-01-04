const crypto = require('crypto')

const ass = `eyJsaWNlbnNlIjp7ImlkIjoidGVzdCIsIm1ldGEiOnt9LCJ2YWxpZF9mcm9tIjoiMjAwMC0xLTEiLCJ2YWxpZF91bnRpbCI6IjMwMDAtMS0xIn0sInNpZ25hdHVyZSI6ImVhYzJkMjI2ZjA0NTFjMmQ5NTM2NzkxZDg2NDEyMjRhZWFmMjkwY2NmZjEzYWQxZDE0YmYxY2U2OGMyYzJmMmQifQ==`

const d = JSON.parse((Buffer.from(ass, 'base64').toString('ascii')));
const parts = [];
parts.push(d['license']['valid_from']);
parts.push(d['license']['valid_until']);
parts.push(d['license']['id']);
Object.values(d['license']['meta']).forEach(v => parts.push(v));

const to_hash = parts.join("\n");

const result = crypto.createHmac('sha256', 'SECRET')
    .update(to_hash)
    .digest('hex')

console.log(result === d['signature'])
