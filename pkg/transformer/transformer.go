package transformer

import "strings"

type EnglishTransformer struct {
}

func NewEnglishTransformer() *EnglishTransformer {
	return &EnglishTransformer{}
}

func (t *EnglishTransformer) Lowercase(tokens []string) []string {

	for idx := range tokens {
		tokens[idx] = strings.ToLower(tokens[idx])
	}
	return tokens
}
