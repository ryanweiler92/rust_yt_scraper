1. `get_comments()`
   - Entry point method
   - Gets `continuation_token` used for requesting main comments
   - Gets `api_key` which is also required for requesting main comments
   - The `comments_data` represents the returned JSON from the http request
2. `comment_extractor()`
   - The entry point method calls this method to extract comments from the requested json
   - Gets the comments section as a list `comment_content_list_actual`
   - Also gets a list of the continuation items section required to request continuations for comment replies
   - We loop through each of the comments returned in the `comment_content_list_actual`
   - First we get the main comment info with `get_comment_info()`
   - Second we check if it has replies and request replies with the continuation token
3. `get_comment_info()`
    - A method for extracting the individual comment info that is used by both main comments and replies
4. `reply_extractor()`

## JSON Files

1. `1_main_comment_response_1.json`
    - The raw main comment response JSON
2. `2_main_comment_content_2.json`
   - This is a subsection of the main comment response JSON.This is the list of actual comments.
3. `3_continuation_items_3.json`
   - A list of continuation sections for each comment.
4. 