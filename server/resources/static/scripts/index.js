function search_button_click() {
    console.log("clicked on the search button");

    search_text_field = document.getElementById("searchBox");

    console.log(search_text_field.value);
    if (search_text_field.value) {
        document.location = "/search?q=" + encodeURIComponent(search_text_field.value);
    }
}

function setup() {
    
    document.querySelector("#searchBox").addEventListener("keyup", event => {
        if(event.key !== "Enter") return;
        document.querySelector("#searchButton").click(); 
        event.preventDefault();
    });

}