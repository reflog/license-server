import base64
import hmac
import hashlib
import json

ass = """eyJsaWNlbnNlIjp7ImlkIjoidGVzdCIsIm1ldGEiOnt9LCJ2YWxpZF9mcm9tIjoiMjAwMC0xLTEiLCJ2YWxpZF91bnRpbCI6IjMwMDAtMS0xIn0sInNpZ25hdHVyZSI6ImVhYzJkMjI2ZjA0NTFjMmQ5NTM2NzkxZDg2NDEyMjRhZWFmMjkwY2NmZjEzYWQxZDE0YmYxY2U2OGMyYzJmMmQifQ== """

d = json.loads(base64.standard_b64decode(ass))

parts = []
parts += [d['license']['valid_from']]
parts += [d['license']['valid_until']]
parts += [d['license']['id']]
for k, v in d['license']['meta']:
    parts += [v]
to_hash = "\n".join(parts)

digest_maker = hmac.new(bytes('SECRET', encoding='utf8'), None, hashlib.sha256)
digest_maker.update(bytes(to_hash, encoding='utf8'))

print(digest_maker.hexdigest() == d['signature'])
