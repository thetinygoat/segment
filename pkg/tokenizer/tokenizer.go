package tokenizer

import "github.com/jdkato/prose/v2"

type EnglishTokenizer struct {
}

func NewEnglishTokenizer() *EnglishTokenizer {
	return &EnglishTokenizer{}
}

func (e *EnglishTokenizer) Tokenize(input string) ([]string, error) {
	doc, err := prose.NewDocument(input, prose.WithExtraction(false), prose.WithTagging(false), prose.WithSegmentation(false))
	if err != nil {
		return nil, err
	}
	var tokens []string
	for _, t := range doc.Tokens() {
		tokens = append(tokens, t.Text)
	}
	return tokens, nil
}
