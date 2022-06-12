from models import Posts, PostTags, Users, Tags

async def insert_question(q: dict, slug: str, user: Users, conn):
    post = Posts(
        title=q.question['title'],
        slug=slug,
        description=q.question['body'],
        posted_by_id=user.id,
        updated_by_id=user.id,
        op_id=user.id
    )
    await post.save(using_db=conn)
    return post

async def update_post_tags(q:dict, post: Posts, conn):
    tags = await Tags.filter(name__in=q.question['tagList']).using_db(conn)
    
    for t in tags:
        topic_tags = PostTags(
            post_id=post.id,
            tag_id=t.id
        )
        await topic_tags.save(using_db=conn)