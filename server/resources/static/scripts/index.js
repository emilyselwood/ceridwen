function search_button_click() {
    console.log("clicked on the search button");

    search_text_field = document.getElementById("query_box");

    console.log(search_text_field.value);
    if (search_text_field.value) {
        document.location = "/search?q=" + encodeURIComponent(search_text_field.value);
    }
}