from itsdangerous import TimestampSigner
from config import settings


async def get_token(arg: str):
    s = TimestampSigner(settings["secret_key"])
    return s.sign(arg)


async def verify_token(arg: str):
    s = TimestampSigner(settings["secret_key"])
    return s.unsign(arg, max_age=settings["email_verification_expiry_time"])
