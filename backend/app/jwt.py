import hmac
import hashlib
import base64
import json
import time


def getSignature(base64Header, base64Payload, secret):
    block = base64Header.decode("utf-8") + "." + base64Payload.decode("utf-8")
    digest = hmac.new(
        bytes(secret, "utf-8"), block.encode("utf-8"), digestmod=hashlib.sha256
    ).digest()
    signature = base64.urlsafe_b64encode(digest)
    return signature.decode("utf-8")[:-1]


def encodeJWT(data, key, algorithm):
    payload = data
    header = {"alg": algorithm, "typ": "JWT"}
    base64Header = base64.b64encode(json.dumps(header).encode("utf-8"))
    # Dumping header and payload dictionaries to string then encoding in bytes and then finally encoding in base64 bytes
    base64Payload = base64.b64encode(json.dumps(payload).encode("utf-8"))
    sig = getSignature(base64Header, base64Payload, key)
    encodedJWT = (
        base64Header.decode("utf-8") + "." + base64Payload.decode("utf-8") + "." + sig
    )
    return encodedJWT


def decodeJWT(access_token, key):
    header = access_token.split(".")[0]
    payload = access_token.split(".")[1]
    decodedPayload = base64.b64decode(payload)
    sig = getSignature(header.encode("utf-8"), payload.encode("utf-8"), key)
    res = {
        "payload": decodedPayload.decode("utf-8"),
        "verified": (sig == access_token.split(".")[2]),
    }
    if sig == access_token.split(".")[2]:
        return res
    else:
        return "Couldn't Verify Signature"
