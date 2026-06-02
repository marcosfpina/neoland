"use client"

import { useState } from "react"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Badge } from "@/components/ui/badge"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Palette, Type, Layers, Copy, Download, Plus, Trash2 } from "lucide-react"

export function DesignSystem() {
  const [colors, setColors] = useState([
    { name: "Primary", value: "#3B82F6", usage: "Buttons, links, highlights" },
    { name: "Secondary", value: "#6B7280", usage: "Secondary text, borders" },
    { name: "Success", value: "#10B981", usage: "Success states, confirmations" },
    { name: "Warning", value: "#F59E0B", usage: "Warnings, alerts" },
    { name: "Error", value: "#EF4444", usage: "Errors, destructive actions" },
    { name: "Background", value: "#F9FAFB", usage: "Page backgrounds" },
    { name: "Surface", value: "#FFFFFF", usage: "Card backgrounds, modals" },
    { name: "Text Primary", value: "#111827", usage: "Headings, primary text" },
    { name: "Text Secondary", value: "#6B7280", usage: "Secondary text, captions" },
  ])

  const [typography, setTypography] = useState([
    { name: "Heading 1", size: "2.25rem", weight: "700", lineHeight: "2.5rem", usage: "Page titles" },
    { name: "Heading 2", size: "1.875rem", weight: "600", lineHeight: "2.25rem", usage: "Section titles" },
    { name: "Heading 3", size: "1.5rem", weight: "600", lineHeight: "2rem", usage: "Subsection titles" },
    { name: "Body Large", size: "1.125rem", weight: "400", lineHeight: "1.75rem", usage: "Large body text" },
    { name: "Body", size: "1rem", weight: "400", lineHeight: "1.5rem", usage: "Default body text" },
    { name: "Body Small", size: "0.875rem", weight: "400", lineHeight: "1.25rem", usage: "Small text, captions" },
    { name: "Caption", size: "0.75rem", weight: "400", lineHeight: "1rem", usage: "Tiny text, labels" },
  ])

  const [spacing, setSpacing] = useState([
    { name: "xs", value: "0.25rem", pixels: "4px" },
    { name: "sm", value: "0.5rem", pixels: "8px" },
    { name: "md", value: "1rem", pixels: "16px" },
    { name: "lg", value: "1.5rem", pixels: "24px" },
    { name: "xl", value: "2rem", pixels: "32px" },
    { name: "2xl", value: "3rem", pixels: "48px" },
    { name: "3xl", value: "4rem", pixels: "64px" },
  ])

  const [components, _setComponents] = useState([
    {
      name: "Primary Button",
      category: "Buttons",
      code: `<button className="px-4 py-2 bg-blue-500 text-white rounded-md hover:bg-blue-600 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2">
  Primary Button
</button>`,
      preview: "🔵 Primary",
    },
    {
      name: "Secondary Button",
      category: "Buttons",
      code: `<button className="px-4 py-2 border border-gray-300 text-gray-700 rounded-md hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2">
  Secondary Button
</button>`,
      preview: "⚪ Secondary",
    },
    {
      name: "Input Field",
      category: "Forms",
      code: `<input 
  type="text" 
  placeholder="Enter text..." 
  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
/>`,
      preview: "📝 Input",
    },
    {
      name: "Card",
      category: "Layout",
      code: `<div className="bg-white rounded-lg shadow-md border border-gray-200 p-6">
  <h3 className="text-lg font-semibold text-gray-900 mb-2">Card Title</h3>
  <p className="text-gray-600">Card content goes here...</p>
</div>`,
      preview: "📄 Card",
    },
  ])

  const exportDesignSystem = () => {
    const designSystem = {
      colors,
      typography,
      spacing,
      components,
      metadata: {
        name: "Design System",
        version: "1.0.0",
        createdAt: new Date().toISOString(),
      },
    }

    const blob = new Blob([JSON.stringify(designSystem, null, 2)], { type: "application/json" })
    const url = URL.createObjectURL(blob)
    const a = document.createElement("a")
    a.href = url
    a.download = "design-system.json"
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
    URL.revokeObjectURL(url)
  }

  const generateTailwindConfig = () => {
    const config = `module.exports = {
  theme: {
    extend: {
      colors: {
${colors.map((color) => `        '${color.name.toLowerCase().replace(/\s+/g, "-")}': '${color.value}',`).join("\n")}
      },
      fontSize: {
${typography
  .map((type) => `        '${type.name.toLowerCase().replace(/\s+/g, "-")}': ['${type.size}', '${type.lineHeight}'],`)
  .join("\n")}
      },
      spacing: {
${spacing.map((space) => `        '${space.name}': '${space.value}',`).join("\n")}
      }
    }
  }
}`

    navigator.clipboard.writeText(config)
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-white">Design System</h2>
          <p className="text-gray-300">Manage your design tokens and component library</p>
        </div>
        <div className="flex gap-2">
          <Button onClick={generateTailwindConfig} variant="outline" className="border-slate-600">
            <Copy className="w-4 h-4 mr-2" />
            Copy Tailwind Config
          </Button>
          <Button onClick={exportDesignSystem} variant="outline" className="border-slate-600">
            <Download className="w-4 h-4 mr-2" />
            Export System
          </Button>
        </div>
      </div>

      <Tabs defaultValue="colors" className="w-full">
        <TabsList className="grid w-full grid-cols-4 bg-slate-800/50">
          <TabsTrigger value="colors" className="data-[state=active]:bg-slate-700">
            Colors
          </TabsTrigger>
          <TabsTrigger value="typography" className="data-[state=active]:bg-slate-700">
            Typography
          </TabsTrigger>
          <TabsTrigger value="spacing" className="data-[state=active]:bg-slate-700">
            Spacing
          </TabsTrigger>
          <TabsTrigger value="components" className="data-[state=active]:bg-slate-700">
            Components
          </TabsTrigger>
        </TabsList>

        <TabsContent value="colors" className="mt-6">
          <Card className="bg-slate-800/50 border-slate-700">
            <CardHeader>
              <div className="flex items-center justify-between">
                <div>
                  <CardTitle className="text-white flex items-center gap-2">
                    <Palette className="w-5 h-5" />
                    Color Palette
                  </CardTitle>
                  <CardDescription className="text-gray-300">Define your brand colors and usage</CardDescription>
                </div>
                <Button variant="outline" size="sm" className="border-slate-600">
                  <Plus className="w-4 h-4 mr-2" />
                  Add Color
                </Button>
              </div>
            </CardHeader>
            <CardContent>
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                {colors.map((color, index) => (
                  <div key={index} className="p-4 bg-slate-700/30 rounded-lg">
                    <div className="flex items-center gap-3 mb-3">
                      <div
                        className="w-12 h-12 rounded-lg border-2 border-white/20"
                        style={{ backgroundColor: color.value }}
                      ></div>
                      <div className="flex-1">
                        <Input
                          value={color.name}
                          onChange={(e) => {
                            const newColors = [...colors]
                            newColors[index].name = e.target.value
                            setColors(newColors)
                          }}
                          className="bg-slate-600 border-slate-500 text-white text-sm font-medium mb-1"
                        />
                        <Input
                          value={color.value}
                          onChange={(e) => {
                            const newColors = [...colors]
                            newColors[index].value = e.target.value
                            setColors(newColors)
                          }}
                          className="bg-slate-600 border-slate-500 text-white text-xs font-mono"
                        />
                      </div>
                      <Button variant="ghost" size="sm" className="text-gray-400 hover:text-red-400">
                        <Trash2 className="w-4 h-4" />
                      </Button>
                    </div>
                    <Input
                      value={color.usage}
                      onChange={(e) => {
                        const newColors = [...colors]
                        newColors[index].usage = e.target.value
                        setColors(newColors)
                      }}
                      placeholder="Usage description..."
                      className="bg-slate-600 border-slate-500 text-white text-xs"
                    />
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="typography" className="mt-6">
          <Card className="bg-slate-800/50 border-slate-700">
            <CardHeader>
              <div className="flex items-center justify-between">
                <div>
                  <CardTitle className="text-white flex items-center gap-2">
                    <Type className="w-5 h-5" />
                    Typography Scale
                  </CardTitle>
                  <CardDescription className="text-gray-300">Define your text styles and hierarchy</CardDescription>
                </div>
                <Button variant="outline" size="sm" className="border-slate-600">
                  <Plus className="w-4 h-4 mr-2" />
                  Add Style
                </Button>
              </div>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                {typography.map((type, index) => (
                  <div key={index} className="p-4 bg-slate-700/30 rounded-lg">
                    <div className="grid grid-cols-1 md:grid-cols-5 gap-4 items-center">
                      <div>
                        <Input
                          value={type.name}
                          onChange={(e) => {
                            const newTypography = [...typography]
                            newTypography[index].name = e.target.value
                            setTypography(newTypography)
                          }}
                          className="bg-slate-600 border-slate-500 text-white text-sm font-medium"
                        />
                      </div>
                      <div>
                        <label className="text-gray-400 text-xs block mb-1">Size</label>
                        <Input
                          value={type.size}
                          onChange={(e) => {
                            const newTypography = [...typography]
                            newTypography[index].size = e.target.value
                            setTypography(newTypography)
                          }}
                          className="bg-slate-600 border-slate-500 text-white text-sm"
                        />
                      </div>
                      <div>
                        <label className="text-gray-400 text-xs block mb-1">Weight</label>
                        <Input
                          value={type.weight}
                          onChange={(e) => {
                            const newTypography = [...typography]
                            newTypography[index].weight = e.target.value
                            setTypography(newTypography)
                          }}
                          className="bg-slate-600 border-slate-500 text-white text-sm"
                        />
                      </div>
                      <div>
                        <label className="text-gray-400 text-xs block mb-1">Line Height</label>
                        <Input
                          value={type.lineHeight}
                          onChange={(e) => {
                            const newTypography = [...typography]
                            newTypography[index].lineHeight = e.target.value
                            setTypography(newTypography)
                          }}
                          className="bg-slate-600 border-slate-500 text-white text-sm"
                        />
                      </div>
                      <div className="flex items-center gap-2">
                        <Input
                          value={type.usage}
                          onChange={(e) => {
                            const newTypography = [...typography]
                            newTypography[index].usage = e.target.value
                            setTypography(newTypography)
                          }}
                          placeholder="Usage..."
                          className="bg-slate-600 border-slate-500 text-white text-sm flex-1"
                        />
                        <Button variant="ghost" size="sm" className="text-gray-400 hover:text-red-400">
                          <Trash2 className="w-4 h-4" />
                        </Button>
                      </div>
                    </div>
                    <div className="mt-3 p-3 bg-slate-600/30 rounded">
                      <div
                        className="text-white"
                        style={{
                          fontSize: type.size,
                          fontWeight: type.weight,
                          lineHeight: type.lineHeight,
                        }}
                      >
                        {type.name} Preview Text
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="spacing" className="mt-6">
          <Card className="bg-slate-800/50 border-slate-700">
            <CardHeader>
              <div className="flex items-center justify-between">
                <div>
                  <CardTitle className="text-white flex items-center gap-2">
                    <Layers className="w-5 h-5" />
                    Spacing Scale
                  </CardTitle>
                  <CardDescription className="text-gray-300">Define consistent spacing values</CardDescription>
                </div>
                <Button variant="outline" size="sm" className="border-slate-600">
                  <Plus className="w-4 h-4 mr-2" />
                  Add Size
                </Button>
              </div>
            </CardHeader>
            <CardContent>
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                {spacing.map((space, index) => (
                  <div key={index} className="p-4 bg-slate-700/30 rounded-lg">
                    <div className="flex items-center gap-3 mb-3">
                      <div className="flex-1">
                        <Input
                          value={space.name}
                          onChange={(e) => {
                            const newSpacing = [...spacing]
                            newSpacing[index].name = e.target.value
                            setSpacing(newSpacing)
                          }}
                          className="bg-slate-600 border-slate-500 text-white text-sm font-medium mb-2"
                        />
                        <Input
                          value={space.value}
                          onChange={(e) => {
                            const newSpacing = [...spacing]
                            newSpacing[index].value = e.target.value
                            setSpacing(newSpacing)
                          }}
                          className="bg-slate-600 border-slate-500 text-white text-sm mb-2"
                        />
                        <Input
                          value={space.pixels}
                          onChange={(e) => {
                            const newSpacing = [...spacing]
                            newSpacing[index].pixels = e.target.value
                            setSpacing(newSpacing)
                          }}
                          className="bg-slate-600 border-slate-500 text-white text-sm"
                        />
                      </div>
                      <Button variant="ghost" size="sm" className="text-gray-400 hover:text-red-400">
                        <Trash2 className="w-4 h-4" />
                      </Button>
                    </div>
                    <div className="flex items-center gap-2">
                      <div
                        className="bg-blue-500 rounded"
                        style={{
                          width: space.value,
                          height: space.value,
                          minWidth: "4px",
                          minHeight: "4px",
                        }}
                      ></div>
                      <span className="text-gray-400 text-xs">{space.pixels}</span>
                    </div>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="components" className="mt-6">
          <Card className="bg-slate-800/50 border-slate-700">
            <CardHeader>
              <div className="flex items-center justify-between">
                <div>
                  <CardTitle className="text-white flex items-center gap-2">
                    <Layers className="w-5 h-5" />
                    Component Library
                  </CardTitle>
                  <CardDescription className="text-gray-300">Reusable UI components and patterns</CardDescription>
                </div>
                <Button variant="outline" size="sm" className="border-slate-600">
                  <Plus className="w-4 h-4 mr-2" />
                  Add Component
                </Button>
              </div>
            </CardHeader>
            <CardContent>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                {components.map((component, index) => (
                  <div key={index} className="p-4 bg-slate-700/30 rounded-lg">
                    <div className="flex items-center justify-between mb-3">
                      <div>
                        <h4 className="text-white font-medium">{component.name}</h4>
                        <Badge variant="outline" className="text-xs border-slate-600 mt-1">
                          {component.category}
                        </Badge>
                      </div>
                      <div className="flex gap-2">
                        <Button variant="ghost" size="sm" className="text-gray-400 hover:text-blue-400">
                          <Copy className="w-4 h-4" />
                        </Button>
                        <Button variant="ghost" size="sm" className="text-gray-400 hover:text-red-400">
                          <Trash2 className="w-4 h-4" />
                        </Button>
                      </div>
                    </div>
                    <div className="mb-3 p-3 bg-slate-600/30 rounded text-center">
                      <span className="text-2xl">{component.preview}</span>
                    </div>
                    <pre className="text-xs text-gray-300 bg-slate-900 p-3 rounded overflow-x-auto">
                      <code>{component.code}</code>
                    </pre>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  )
}
