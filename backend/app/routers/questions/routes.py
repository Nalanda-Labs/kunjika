from fastapi import APIRouter, HTTPException, Depends
from fastapi.responses import ORJSONResponse

from tortoise.transactions import atomic, in_transaction

import security
import app.routers.questions.schemas as schemas
from routers.questions.crud import insert_question, update_post_tags
from routers.users.crud import get_user_by_email
from routers.tags.crud import verify_tags, update_tags_count


router = APIRouter(prefix="/api")
import logging

fmt = logging.Formatter(
    fmt="%(asctime)s - %(name)s:%(lineno)d - %(levelname)s - %(message)s",
    datefmt="%Y-%m-%d %H:%M:%S",
)
import sys
sh = logging.StreamHandler(sys.stdout)
sh.setLevel(logging.DEBUG)
sh.setFormatter(fmt)

# will print debug sql
logger_db_client = logging.getLogger("db_client")
logger_db_client.setLevel(logging.DEBUG)
logger_db_client.addHandler(sh)

logger_tortoise = logging.getLogger("tortoise")
logger_tortoise.setLevel(logging.DEBUG)
logger_tortoise.addHandler(sh)

@atomic()
@router.post("/create-question/")
async def create_question(
    q: schemas.Question,
    response_class=ORJSONResponse,
    email=Depends(security.verify_user),
):
    if not email:
        raise HTTPException(status_code=401, detail="Unauthorized")

    if len(q.question["title"]) < 10 or len(q.question["title"]) > 256:
        raise HTTPException(status_code=422, detail="Title too long or too small!")
    if len(q.question["body"]) < 20 or len(q.question["body"]) > 1000000:
        raise HTTPException(status_code=422, detail="Body too long or too small!")
    if len(q.question["tagList"]) < 1 or len(q.question["tagList"]) > 5:
        raise HTTPException(
            status_code=422,
            detail="There should be at least 1 or at most 5 tags must be supplied",
        )

    tags = []
    for t in q.question["tagList"]:
        if len(t) > 32:
            raise HTTPException(
                status_code=422, detail="Maximum tag length is 32 bytes!"
            )
        tags.append(t.lower())

    q.question["tagList"] = tags

    if not await verify_tags(q.question["tagList"]):
        raise HTTPException(status_code=422, detail="Some tags were not found!")

    slug = ""
    prev_dash = False

    for c in q.question["title"]:
        if (c >= "a" and c <= "z") or (c >= "0" and c <= "9"):
            slug += c
            prev_dash = False
        elif c >= "A" and c <= "Z":
            slug += c
            prev_dash = False
        elif (
            c == " "
            or c == ","
            or c == "."
            or c == "/"
            or c == "\\"
            or c == "-"
            or c == "_"
            or c == "="
        ):
            if (not prev_dash) and (len(slug) > 0):
                slug += "-"
                prev_dash = True
    if prev_dash:
        slug = slug[:-1]

    async with in_transaction() as connection:
        user = await get_user_by_email(email)

        await update_tags_count(q.question["tagList"], connection)
        post = await insert_question(q, slug, user, connection)
        await update_post_tags(q, post, connection)
        # need to serialize id to string because JSON cannot handle 64-bit integers
        return {"id": str(post.id), "slug": slug}
