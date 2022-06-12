from typing import List

from tortoise import Tortoise

from models import Tags


async def get_tags_like(tag: str):
    tags = await Tags.filter(name__startswith=tag).values_list("name", flat=True)
    return {"tags": tags}


async def verify_tags(tags: List[str]):
    db_tags = await Tags.filter(name__in=tags).values_list("name", flat=True)
    for t in tags:
        if t not in db_tags:
            return False
    return True


async def update_tags_count(tags: List[str], conn):
    # cannot do this in one query in orm
    await conn.execute_query(
        "update tags set post_count=post_count + 1 where name in " + str(tuple(tags))
    )
