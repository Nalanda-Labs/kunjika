from models import Tags

async def get_tags_like(tag):
    tags = await Tags.filter(name__startswith=tag).values_list('name', flat=True)
    print(tags)
    return {"tags": tags}