package main

import (
	"encoding/json"
	"net/http"
	"os"

	"github.com/google/uuid"
)

const permissions = 0700

type PayLoad struct {
	TestCases string `json:"testCases"`
	Solution  string `json:"solution"`
}

func main() {
	http.HandleFunc("POST /task", task)
	http.ListenAndServe(":8080", nil)
}

func task(w http.ResponseWriter, r *http.Request) {
	var task PayLoad
	err := json.NewDecoder(r.Body).Decode(&task)
	if err != nil {
		http.Error(w, "could not decode json payload", http.StatusBadRequest)
		return
	}

	// you would probably make a helper function to create necessary structure
	id := uuid.New()
	err = os.Mkdir(id.String(), permissions)
	if err != nil {
		http.Error(w, "could not create necessary test structure", http.StatusInternalServerError)
		return
	}

	// so on
}
