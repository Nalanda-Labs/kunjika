import hashlib
import base64
from datetime import datetime, timedelta
from typing import Optional
from jwt import *
import json
import crud

SECRET_KEY = ""
ALGORITHM = "HS256"




def verify_hash(password,savedSalt):
    # Salt is in utf-8 string I need to encode it in Base64 and then decode the Base64 to bytes
    savedSalt = savedSalt.encode('utf-8')
    savedSalt = base64.b64decode(savedSalt)
    key = hashlib.pbkdf2_hmac(
    'sha256', # The hash digest algorithm for HMAC
    password.encode('utf-8'), # Convert the password to bytes
    savedSalt, # Provide the salt
    100000 # It is recommended to use at least 100,000 iterations of SHA-256 
)   
    key = base64.b64encode(key)
    return key


def create_access_token(data: dict, expires_delta: Optional[timedelta] = None):
    to_encode = data.copy()
    if expires_delta:
        expire = datetime.utcnow() + expires_delta
    else:
        expire = datetime.utcnow() + timedelta(minutes=15)

    to_encode.update({"exp": expire.isoformat()})
    encoded_jwt = encodeJWT(to_encode, SECRET_KEY, algorithm=ALGORITHM)
    return encoded_jwt


def get_current_user_email(token):
    decoded = decodeJWT(token, SECRET_KEY)
    # email: str = payload["sub"]
    user_email = json.loads(decoded["payload"])["sub"]
    return user_email
    # if username is None:
    #     raise credentials_exception
    # token_data = TokenData(username=username)
    # except JWTError:
    #     raise credentials_exception
    # user = get_user(fake_users_db, username=token_data.username)
    # if user is None:
    #     raise credentials_exception
    # return user
