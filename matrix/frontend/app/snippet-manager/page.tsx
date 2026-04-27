"use client"

import { useState } from "react"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Textarea } from "@/components/ui/textarea"
import { Badge } from "@/components/ui/badge"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Code2, Search, Plus, Copy, Star, Tag, ArrowLeft, Brain, FileText, Folder, Heart, Share } from "lucide-react"
import Link from "next/link"

export default function SnippetManager() {
  const [searchQuery, setSearchQuery] = useState("")
  const [selectedLanguage, setSelectedLanguage] = useState("all")
  const [selectedCategory, setSelectedCategory] = useState("all")
  const [newSnippet, setNewSnippet] = useState({
    title: "",
    code: "",
    language: "javascript",
    description: "",
    tags: "",
  })

  const snippets = [
    {
      id: 1,
      title: "React useLocalStorage Hook",
      code: `const useLocalStorage = (key, initialValue) => {
  const [storedValue, setStoredValue] = useState(() => {
    try {
      const item = window.localStorage.getItem(key);
      return item ? JSON.parse(item) : initialValue;
    } catch (error) {
      return initialValue;
    }
  });

  const setValue = (value) => {
    try {
      setStoredValue(value);
      window.localStorage.setItem(key, JSON.stringify(value));
    } catch (error) {
      console.error(error);
    }
  };

  return [storedValue, setValue];
};`,
      language: "javascript",
      category: "React Hooks",
      tags: ["react", "hooks", "localStorage", "state"],
      description: "Custom hook for managing localStorage with React state",
      starred: true,
      likes: 24,
      createdAt: "2024-01-15",
    },
    {
      id: 2,
      title: "Python Data Validation Decorator",
      code: `def validate_types(**expected_types):
    def decorator(func):
        def wrapper(*args, **kwargs):
            # Validate positional arguments
            for i, (arg, expected_type) in enumerate(zip(args, expected_types.values())):
                if not isinstance(arg, expected_type):
                    raise TypeError(f"Argument {i} must be of type {expected_type.__name__}")
            
            # Validate keyword arguments
            for key, value in kwargs.items():
                if key in expected_types and not isinstance(value, expected_types[key]):
                    raise TypeError(f"Argument '{key}' must be of type {expected_types[key].__name__}")
            
            return func(*args, **kwargs)
        return wrapper
    return decorator

# Usage
@validate_types(name=str, age=int)
def create_user(name, age):
    return {"name": name, "age": age}`,
      language: "python",
      category: "Utilities",
      tags: ["python", "decorator", "validation", "types"],
      description: "Decorator for runtime type validation in Python functions",
      starred: false,
      likes: 18,
      createdAt: "2024-01-14",
    },
    {
      id: 3,
      title: "CSS Flexbox Center",
      code: `.flex-center {
  display: flex;
  justify-content: center;
  align-items: center;
}

.flex-center-column {
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
}

.flex-space-between {
  display: flex;
  justify-content: space-between;
  align-items: center;
}`,
      language: "css",
      category: "CSS Utilities",
      tags: ["css", "flexbox", "layout", "center"],
      description: "Common flexbox patterns for centering and layout",
      starred: true,
      likes: 31,
      createdAt: "2024-01-13",
    },
  ]

  const categories = ["All", "React Hooks", "Utilities", "CSS Utilities", "API Helpers", "Database Queries"]
  const languages = ["All", "JavaScript", "TypeScript", "Python", "CSS", "SQL", "Go", "Rust"]

  const filteredSnippets = snippets.filter((snippet) => {
    const matchesSearch =
      snippet.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
      snippet.description.toLowerCase().includes(searchQuery.toLowerCase()) ||
      snippet.tags.some((tag) => tag.toLowerCase().includes(searchQuery.toLowerCase()))

    const matchesLanguage =
      selectedLanguage === "all" || snippet.language.toLowerCase() === selectedLanguage.toLowerCase()
    const matchesCategory = selectedCategory === "all" || snippet.category === selectedCategory

    return matchesSearch && matchesLanguage && matchesCategory
  })

  const handleCopySnippet = (code) => {
    navigator.clipboard.writeText(code)
    // You could add a toast notification here
  }

  const generateSnippetFromDescription = () => {
    // Simulate AI code generation
    const generatedCode = `// AI Generated Code
function ${newSnippet.title.replace(/\s+/g, "")}() {
  // Implementation based on: ${newSnippet.description}
  console.log('Generated function');
  return true;
}`
    setNewSnippet({ ...newSnippet, code: generatedCode })
  }

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-900 via-indigo-900 to-slate-900">
      <div className="container mx-auto px-4 py-8">
        {/* Header */}
        <div className="flex items-center gap-4 mb-8">
          <Link href="/">
            <Button variant="outline" size="sm" className="border-slate-600">
              <ArrowLeft className="w-4 h-4 mr-2" />
              Back to Hub
            </Button>
          </Link>
          <div className="flex items-center gap-3">
            <div className="p-2 bg-indigo-500 rounded-lg">
              <Code2 className="w-6 h-6 text-white" />
            </div>
            <div>
              <h1 className="text-3xl font-bold text-white">Code Snippet Hub</h1>
              <p className="text-gray-300">AI-categorized code snippets with intelligent search</p>
            </div>
          </div>
        </div>

        <div className="grid lg:grid-cols-4 gap-8">
          {/* Sidebar */}
          <Card className="bg-slate-800/50 border-slate-700">
            <CardHeader>
              <CardTitle className="text-white flex items-center gap-2">
                <Folder className="w-5 h-5" />
                Filters
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              {/* Search */}
              <div>
                <label className="text-white text-sm font-medium mb-2 block">Search</label>
                <div className="relative">
                  <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-gray-400" />
                  <Input
                    value={searchQuery}
                    onChange={(e) => setSearchQuery(e.target.value)}
                    placeholder="Search snippets..."
                    className="pl-10 bg-slate-700 border-slate-600 text-white"
                  />
                </div>
              </div>

              {/* Language Filter */}
              <div>
                <label className="text-white text-sm font-medium mb-2 block">Language</label>
                <Select value={selectedLanguage} onValueChange={setSelectedLanguage}>
                  <SelectTrigger className="bg-slate-700 border-slate-600">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {languages.map((lang) => (
                      <SelectItem key={lang.toLowerCase()} value={lang.toLowerCase()}>
                        {lang}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>

              {/* Category Filter */}
              <div>
                <label className="text-white text-sm font-medium mb-2 block">Category</label>
                <Select value={selectedCategory} onValueChange={setSelectedCategory}>
                  <SelectTrigger className="bg-slate-700 border-slate-600">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {categories.map((category) => (
                      <SelectItem key={category} value={category}>
                        {category}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>

              {/* Quick Stats */}
              <div className="pt-4 border-t border-slate-600">
                <h4 className="text-white font-medium mb-2">Quick Stats</h4>
                <div className="space-y-1 text-sm">
                  <div className="flex justify-between text-gray-300">
                    <span>Total Snippets</span>
                    <span>{snippets.length}</span>
                  </div>
                  <div className="flex justify-between text-gray-300">
                    <span>Starred</span>
                    <span>{snippets.filter((s) => s.starred).length}</span>
                  </div>
                  <div className="flex justify-between text-gray-300">
                    <span>Languages</span>
                    <span>{new Set(snippets.map((s) => s.language)).size}</span>
                  </div>
                </div>
              </div>
            </CardContent>
          </Card>

          {/* Main Content */}
          <div className="lg:col-span-3">
            <Tabs defaultValue="browse" className="w-full">
              <TabsList className="grid w-full grid-cols-3 bg-slate-800/50">
                <TabsTrigger value="browse" className="data-[state=active]:bg-slate-700">
                  Browse Snippets
                </TabsTrigger>
                <TabsTrigger value="create" className="data-[state=active]:bg-slate-700">
                  Create New
                </TabsTrigger>
                <TabsTrigger value="favorites" className="data-[state=active]:bg-slate-700">
                  Favorites
                </TabsTrigger>
              </TabsList>

              <TabsContent value="browse" className="mt-6">
                <div className="space-y-4">
                  {filteredSnippets.length === 0 ? (
                    <Card className="bg-slate-800/50 border-slate-700">
                      <CardContent className="text-center py-12">
                        <Search className="w-12 h-12 text-gray-500 mx-auto mb-4" />
                        <p className="text-gray-400">No snippets found matching your criteria</p>
                      </CardContent>
                    </Card>
                  ) : (
                    filteredSnippets.map((snippet) => (
                      <Card key={snippet.id} className="bg-slate-800/50 border-slate-700">
                        <CardHeader>
                          <div className="flex items-start justify-between">
                            <div>
                              <CardTitle className="text-white flex items-center gap-2">
                                {snippet.starred && <Star className="w-4 h-4 text-yellow-500 fill-current" />}
                                {snippet.title}
                              </CardTitle>
                              <CardDescription className="text-gray-300 mt-1">{snippet.description}</CardDescription>
                            </div>
                            <div className="flex items-center gap-2">
                              <Button
                                variant="outline"
                                size="sm"
                                onClick={() => handleCopySnippet(snippet.code)}
                                className="border-slate-600"
                              >
                                <Copy className="w-4 h-4" />
                              </Button>
                              <Button variant="outline" size="sm" className="border-slate-600">
                                <Share className="w-4 h-4" />
                              </Button>
                            </div>
                          </div>
                          <div className="flex items-center gap-2 mt-2">
                            <Badge variant="secondary">{snippet.language}</Badge>
                            <Badge variant="outline" className="border-slate-600">
                              {snippet.category}
                            </Badge>
                            <div className="flex items-center gap-1 text-gray-400 text-sm ml-auto">
                              <Heart className="w-4 h-4" />
                              <span>{snippet.likes}</span>
                            </div>
                          </div>
                        </CardHeader>
                        <CardContent>
                          <pre className="bg-slate-900 border border-slate-600 rounded-lg p-4 text-sm text-white overflow-auto max-h-64">
                            <code>{snippet.code}</code>
                          </pre>
                          <div className="flex flex-wrap gap-1 mt-3">
                            {snippet.tags.map((tag) => (
                              <Badge key={tag} variant="outline" className="text-xs border-slate-600">
                                <Tag className="w-3 h-3 mr-1" />
                                {tag}
                              </Badge>
                            ))}
                          </div>
                        </CardContent>
                      </Card>
                    ))
                  )}
                </div>
              </TabsContent>

              <TabsContent value="create" className="mt-6">
                <Card className="bg-slate-800/50 border-slate-700">
                  <CardHeader>
                    <CardTitle className="text-white flex items-center gap-2">
                      <Plus className="w-5 h-5" />
                      Create New Snippet
                    </CardTitle>
                    <CardDescription className="text-gray-300">
                      Add a new code snippet to your collection
                    </CardDescription>
                  </CardHeader>
                  <CardContent className="space-y-4">
                    <div className="grid grid-cols-2 gap-4">
                      <div>
                        <label className="text-white text-sm font-medium mb-2 block">Title</label>
                        <Input
                          value={newSnippet.title}
                          onChange={(e) => setNewSnippet({ ...newSnippet, title: e.target.value })}
                          placeholder="Snippet title"
                          className="bg-slate-700 border-slate-600 text-white"
                        />
                      </div>
                      <div>
                        <label className="text-white text-sm font-medium mb-2 block">Language</label>
                        <Select
                          value={newSnippet.language}
                          onValueChange={(value) => setNewSnippet({ ...newSnippet, language: value })}
                        >
                          <SelectTrigger className="bg-slate-700 border-slate-600">
                            <SelectValue />
                          </SelectTrigger>
                          <SelectContent>
                            <SelectItem value="javascript">JavaScript</SelectItem>
                            <SelectItem value="typescript">TypeScript</SelectItem>
                            <SelectItem value="python">Python</SelectItem>
                            <SelectItem value="css">CSS</SelectItem>
                            <SelectItem value="sql">SQL</SelectItem>
                          </SelectContent>
                        </Select>
                      </div>
                    </div>

                    <div>
                      <label className="text-white text-sm font-medium mb-2 block">Description</label>
                      <Input
                        value={newSnippet.description}
                        onChange={(e) => setNewSnippet({ ...newSnippet, description: e.target.value })}
                        placeholder="What does this snippet do?"
                        className="bg-slate-700 border-slate-600 text-white"
                      />
                    </div>

                    <div>
                      <div className="flex items-center justify-between mb-2">
                        <label className="text-white text-sm font-medium">Code</label>
                        <Button
                          variant="outline"
                          size="sm"
                          onClick={generateSnippetFromDescription}
                          className="border-slate-600"
                        >
                          <Brain className="w-4 h-4 mr-1" />
                          AI Generate
                        </Button>
                      </div>
                      <Textarea
                        value={newSnippet.code}
                        onChange={(e) => setNewSnippet({ ...newSnippet, code: e.target.value })}
                        placeholder="Paste your code here..."
                        className="min-h-48 bg-slate-700 border-slate-600 text-white font-mono"
                      />
                    </div>

                    <div>
                      <label className="text-white text-sm font-medium mb-2 block">Tags</label>
                      <Input
                        value={newSnippet.tags}
                        onChange={(e) => setNewSnippet({ ...newSnippet, tags: e.target.value })}
                        placeholder="react, hooks, state (comma separated)"
                        className="bg-slate-700 border-slate-600 text-white"
                      />
                    </div>

                    <div className="flex gap-2">
                      <Button disabled={!newSnippet.title || !newSnippet.code}>
                        <Plus className="w-4 h-4 mr-2" />
                        Save Snippet
                      </Button>
                      <Button variant="outline" className="border-slate-600">
                        <FileText className="w-4 h-4 mr-2" />
                        Save as Template
                      </Button>
                    </div>
                  </CardContent>
                </Card>
              </TabsContent>

              <TabsContent value="favorites" className="mt-6">
                <div className="space-y-4">
                  {snippets
                    .filter((s) => s.starred)
                    .map((snippet) => (
                      <Card key={snippet.id} className="bg-slate-800/50 border-slate-700">
                        <CardHeader>
                          <CardTitle className="text-white flex items-center gap-2">
                            <Star className="w-4 h-4 text-yellow-500 fill-current" />
                            {snippet.title}
                          </CardTitle>
                          <CardDescription className="text-gray-300">{snippet.description}</CardDescription>
                        </CardHeader>
                        <CardContent>
                          <div className="flex items-center gap-2 mb-3">
                            <Badge variant="secondary">{snippet.language}</Badge>
                            <Badge variant="outline" className="border-slate-600">
                              {snippet.category}
                            </Badge>
                          </div>
                          <pre className="bg-slate-900 border border-slate-600 rounded-lg p-4 text-sm text-white overflow-auto max-h-32">
                            <code>{snippet.code.substring(0, 200)}...</code>
                          </pre>
                        </CardContent>
                      </Card>
                    ))}
                </div>
              </TabsContent>
            </Tabs>
          </div>
        </div>
      </div>
    </div>
  )
}
