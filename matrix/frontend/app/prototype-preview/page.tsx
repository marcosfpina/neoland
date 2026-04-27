"use client"

import { useState, useRef } from "react"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Textarea } from "@/components/ui/textarea"
import { Input } from "@/components/ui/input"
import { Badge } from "@/components/ui/badge"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Slider } from "@/components/ui/slider"
import {
  Eye,
  Code2,
  Smartphone,
  Tablet,
  Monitor,
  ArrowLeft,
  Play,
  Pause,
  RotateCcw,
  Download,
  Share,
  Palette,
  Layout,
  Type,
  Zap,
  Brain,
  Settings,
  Copy,
  Save,
  Layers,
  Grid,
  MousePointer,
  Move,
  Square,
  Circle,
  ImageIcon,
} from "lucide-react"
import Link from "next/link"

export default function PrototypePreview() {
  const [selectedDevice, setSelectedDevice] = useState("desktop")
  const [previewMode, setPreviewMode] = useState("design")
  const [selectedComponent, setSelectedComponent] = useState(null)
  const [components, setComponents] = useState([])
  const [isPlaying, setIsPlaying] = useState(false)
  const [zoom, setZoom] = useState([100])
  const [gridVisible, setGridVisible] = useState(true)
  const [selectedTool, setSelectedTool] = useState("select")
  const canvasRef = useRef(null)

  const devices = {
    mobile: { width: 375, height: 667, name: "iPhone SE", icon: Smartphone },
    tablet: { width: 768, height: 1024, name: "iPad", icon: Tablet },
    desktop: { width: 1440, height: 900, name: "Desktop", icon: Monitor },
  }

  const tools = [
    { id: "select", name: "Select", icon: MousePointer },
    { id: "move", name: "Move", icon: Move },
    { id: "rectangle", name: "Rectangle", icon: Square },
    { id: "circle", name: "Circle", icon: Circle },
    { id: "text", name: "Text", icon: Type },
    { id: "image", name: "Image", icon: ImageIcon },
  ]

  const componentLibrary = [
    {
      id: "button",
      name: "Button",
      category: "Interactive",
      code: `<button className="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600">
  Click me
</button>`,
      preview: "🔘 Button",
    },
    {
      id: "card",
      name: "Card",
      category: "Layout",
      code: `<div className="p-6 bg-white rounded-lg shadow-md border">
  <h3 className="text-lg font-semibold mb-2">Card Title</h3>
  <p className="text-gray-600">Card content goes here...</p>
</div>`,
      preview: "📄 Card",
    },
    {
      id: "input",
      name: "Input Field",
      category: "Forms",
      code: `<input 
  type="text" 
  placeholder="Enter text..." 
  className="w-full px-3 py-2 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
/>`,
      preview: "📝 Input",
    },
    {
      id: "navbar",
      name: "Navigation Bar",
      category: "Navigation",
      code: `<nav className="flex items-center justify-between p-4 bg-white shadow-sm border-b">
  <div className="text-xl font-bold">Logo</div>
  <div className="flex space-x-6">
    <a href="#" className="text-gray-600 hover:text-gray-900">Home</a>
    <a href="#" className="text-gray-600 hover:text-gray-900">About</a>
    <a href="#" className="text-gray-600 hover:text-gray-900">Contact</a>
  </div>
</nav>`,
      preview: "🧭 Navbar",
    },
    {
      id: "hero",
      name: "Hero Section",
      category: "Sections",
      code: `<section className="text-center py-20 bg-gradient-to-r from-blue-500 to-purple-600 text-white">
  <h1 className="text-4xl font-bold mb-4">Welcome to Our App</h1>
  <p className="text-xl mb-8">Build amazing things with our platform</p>
  <button className="px-8 py-3 bg-white text-blue-600 rounded-lg font-semibold hover:bg-gray-100">
    Get Started
  </button>
</section>`,
      preview: "🎯 Hero",
    },
    {
      id: "modal",
      name: "Modal Dialog",
      category: "Overlays",
      code: `<div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center">
  <div className="bg-white p-6 rounded-lg max-w-md w-full mx-4">
    <h2 className="text-xl font-semibold mb-4">Modal Title</h2>
    <p className="text-gray-600 mb-6">Modal content goes here...</p>
    <div className="flex justify-end space-x-2">
      <button className="px-4 py-2 text-gray-600 hover:bg-gray-100 rounded">Cancel</button>
      <button className="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600">Confirm</button>
    </div>
  </div>
</div>`,
      preview: "🪟 Modal",
    },
  ]

  const [currentPrototype, setCurrentPrototype] = useState({
    name: "My App Prototype",
    description: "A modern web application prototype",
    components: [
      {
        id: "nav-1",
        type: "navbar",
        x: 0,
        y: 0,
        width: "100%",
        height: 64,
        props: {},
      },
      {
        id: "hero-1",
        type: "hero",
        x: 0,
        y: 64,
        width: "100%",
        height: 400,
        props: {},
      },
    ],
  })

  const generateAIPrototype = async () => {
    // Simulate AI prototype generation
    const aiPrototypes = [
      {
        name: "E-commerce Dashboard",
        description: "Modern e-commerce admin dashboard with analytics",
        components: [
          { id: "nav-1", type: "navbar", x: 0, y: 0, width: "100%", height: 64 },
          { id: "sidebar-1", type: "sidebar", x: 0, y: 64, width: 240, height: "calc(100% - 64px)" },
          { id: "stats-1", type: "stats", x: 240, y: 64, width: "calc(100% - 240px)", height: 200 },
          { id: "chart-1", type: "chart", x: 240, y: 264, width: "50%", height: 300 },
          { id: "table-1", type: "table", x: "50%", y: 264, width: "50%", height: 300 },
        ],
      },
      {
        name: "Social Media App",
        description: "Mobile-first social media application interface",
        components: [
          { id: "header-1", type: "header", x: 0, y: 0, width: "100%", height: 60 },
          { id: "feed-1", type: "feed", x: 0, y: 60, width: "100%", height: "calc(100% - 120px)" },
          { id: "bottom-nav-1", type: "bottom-nav", x: 0, y: "calc(100% - 60px)", width: "100%", height: 60 },
        ],
      },
      {
        name: "Landing Page",
        description: "Modern SaaS landing page with conversion focus",
        components: [
          { id: "nav-1", type: "navbar", x: 0, y: 0, width: "100%", height: 64 },
          { id: "hero-1", type: "hero", x: 0, y: 64, width: "100%", height: 500 },
          { id: "features-1", type: "features", x: 0, y: 564, width: "100%", height: 400 },
          { id: "pricing-1", type: "pricing", x: 0, y: 964, width: "100%", height: 600 },
          { id: "footer-1", type: "footer", x: 0, y: 1564, width: "100%", height: 200 },
        ],
      },
    ]

    const randomPrototype = aiPrototypes[Math.floor(Math.random() * aiPrototypes.length)]
    setCurrentPrototype(randomPrototype)
  }

  const exportPrototype = () => {
    const exportData = {
      prototype: currentPrototype,
      timestamp: new Date().toISOString(),
      device: selectedDevice,
      zoom: zoom[0],
    }

    const blob = new Blob([JSON.stringify(exportData, null, 2)], { type: "application/json" })
    const url = URL.createObjectURL(blob)
    const a = document.createElement("a")
    a.href = url
    a.download = `${currentPrototype.name.replace(/\s+/g, "-").toLowerCase()}-prototype.json`
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
    URL.revokeObjectURL(url)
  }

  const currentDevice = devices[selectedDevice]

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
              <Eye className="w-6 h-6 text-white" />
            </div>
            <div>
              <h1 className="text-3xl font-bold text-white">Prototype Preview</h1>
              <p className="text-gray-300">Design, preview, and iterate on UI prototypes with AI assistance</p>
            </div>
          </div>
        </div>

        <div className="grid lg:grid-cols-5 gap-6">
          {/* Left Sidebar - Tools & Components */}
          <div className="lg:col-span-1 space-y-6">
            {/* Tools */}
            <Card className="bg-slate-800/50 border-slate-700">
              <CardHeader>
                <CardTitle className="text-white flex items-center gap-2">
                  <Settings className="w-5 h-5" />
                  Tools
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="grid grid-cols-2 gap-2">
                  {tools.map((tool) => {
                    const IconComponent = tool.icon
                    return (
                      <Button
                        key={tool.id}
                        variant={selectedTool === tool.id ? "default" : "outline"}
                        size="sm"
                        onClick={() => setSelectedTool(tool.id)}
                        className="flex flex-col gap-1 h-auto p-2 border-slate-600"
                      >
                        <IconComponent className="w-4 h-4" />
                        <span className="text-xs">{tool.name}</span>
                      </Button>
                    )
                  })}
                </div>
              </CardContent>
            </Card>

            {/* Component Library */}
            <Card className="bg-slate-800/50 border-slate-700">
              <CardHeader>
                <CardTitle className="text-white flex items-center gap-2">
                  <Layers className="w-5 h-5" />
                  Components
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-2">
                  {componentLibrary.map((component) => (
                    <div
                      key={component.id}
                      className="p-2 bg-slate-700/30 rounded cursor-pointer hover:bg-slate-700/50 transition-colors"
                      draggable
                      onDragStart={(e) => {
                        e.dataTransfer.setData("component", JSON.stringify(component))
                      }}
                    >
                      <div className="flex items-center gap-2">
                        <span className="text-lg">{component.preview}</span>
                        <div>
                          <div className="text-white text-sm font-medium">{component.name}</div>
                          <div className="text-gray-400 text-xs">{component.category}</div>
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>

            {/* AI Actions */}
            <Card className="bg-slate-800/50 border-slate-700">
              <CardHeader>
                <CardTitle className="text-white flex items-center gap-2">
                  <Brain className="w-5 h-5" />
                  AI Actions
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-2">
                <Button onClick={generateAIPrototype} variant="outline" size="sm" className="w-full border-slate-600">
                  <Zap className="w-4 h-4 mr-2" />
                  Generate Prototype
                </Button>
                <Button variant="outline" size="sm" className="w-full border-slate-600">
                  <Palette className="w-4 h-4 mr-2" />
                  Suggest Colors
                </Button>
                <Button variant="outline" size="sm" className="w-full border-slate-600">
                  <Layout className="w-4 h-4 mr-2" />
                  Optimize Layout
                </Button>
              </CardContent>
            </Card>
          </div>

          {/* Main Canvas Area */}
          <div className="lg:col-span-3">
            <Card className="bg-slate-800/50 border-slate-700">
              <CardHeader>
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-4">
                    <CardTitle className="text-white">{currentPrototype.name}</CardTitle>
                    <Badge variant="outline" className="border-slate-600">
                      {currentDevice.name}
                    </Badge>
                  </div>
                  <div className="flex items-center gap-2">
                    {/* Device Selection */}
                    <div className="flex items-center gap-1 bg-slate-700/50 rounded p-1">
                      {Object.entries(devices).map(([key, device]) => {
                        const IconComponent = device.icon
                        return (
                          <Button
                            key={key}
                            variant={selectedDevice === key ? "default" : "ghost"}
                            size="sm"
                            onClick={() => setSelectedDevice(key)}
                            className="p-2"
                          >
                            <IconComponent className="w-4 h-4" />
                          </Button>
                        )
                      })}
                    </div>

                    {/* Zoom Control */}
                    <div className="flex items-center gap-2 bg-slate-700/50 rounded px-3 py-1">
                      <span className="text-white text-sm">{zoom[0]}%</span>
                      <Slider value={zoom} onValueChange={setZoom} max={200} min={25} step={25} className="w-20" />
                    </div>

                    {/* View Controls */}
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => setGridVisible(!gridVisible)}
                      className="border-slate-600"
                    >
                      <Grid className="w-4 h-4" />
                    </Button>
                  </div>
                </div>
              </CardHeader>
              <CardContent>
                {/* Canvas */}
                <div className="relative bg-gray-100 rounded-lg overflow-hidden" style={{ height: "600px" }}>
                  {/* Grid Background */}
                  {gridVisible && (
                    <div
                      className="absolute inset-0 opacity-20"
                      style={{
                        backgroundImage: `
                          linear-gradient(to right, #e5e7eb 1px, transparent 1px),
                          linear-gradient(to bottom, #e5e7eb 1px, transparent 1px)
                        `,
                        backgroundSize: "20px 20px",
                      }}
                    />
                  )}

                  {/* Device Frame */}
                  <div
                    className="relative mx-auto bg-white shadow-lg rounded-lg overflow-hidden"
                    style={{
                      width: `${(currentDevice.width * zoom[0]) / 100}px`,
                      height: `${(currentDevice.height * zoom[0]) / 100}px`,
                      maxWidth: "100%",
                      maxHeight: "100%",
                      transform: "translateY(50%)",
                      top: "50%",
                      marginTop: `${-(currentDevice.height * zoom[0]) / 200}px`,
                    }}
                    ref={canvasRef}
                    onDrop={(e) => {
                      e.preventDefault()
                      const componentData = JSON.parse(e.dataTransfer.getData("component"))
                      const rect = canvasRef.current.getBoundingClientRect()
                      const x = e.clientX - rect.left
                      const y = e.clientY - rect.top

                      const newComponent = {
                        id: `${componentData.id}-${Date.now()}`,
                        type: componentData.id,
                        x: (x * 100) / zoom[0],
                        y: (y * 100) / zoom[0],
                        width: 200,
                        height: 100,
                        props: {},
                      }

                      setCurrentPrototype((prev) => ({
                        ...prev,
                        components: [...prev.components, newComponent],
                      }))
                    }}
                    onDragOver={(e) => e.preventDefault()}
                  >
                    {/* Render Components */}
                    {currentPrototype.components.map((component) => (
                      <div
                        key={component.id}
                        className={`absolute border-2 ${
                          selectedComponent?.id === component.id ? "border-blue-500" : "border-transparent"
                        } hover:border-blue-300 cursor-pointer`}
                        style={{
                          left: typeof component.x === "string" ? component.x : `${component.x}px`,
                          top: typeof component.y === "string" ? component.y : `${component.y}px`,
                          width: typeof component.width === "string" ? component.width : `${component.width}px`,
                          height: typeof component.height === "string" ? component.height : `${component.height}px`,
                          transform: `scale(${zoom[0] / 100})`,
                          transformOrigin: "top left",
                        }}
                        onClick={() => setSelectedComponent(component)}
                      >
                        <ComponentRenderer component={component} />
                      </div>
                    ))}

                    {/* Selection Handles */}
                    {selectedComponent && (
                      <div
                        className="absolute border-2 border-blue-500 pointer-events-none"
                        style={{
                          left:
                            typeof selectedComponent.x === "string" ? selectedComponent.x : `${selectedComponent.x}px`,
                          top:
                            typeof selectedComponent.y === "string" ? selectedComponent.y : `${selectedComponent.y}px`,
                          width:
                            typeof selectedComponent.width === "string"
                              ? selectedComponent.width
                              : `${selectedComponent.width}px`,
                          height:
                            typeof selectedComponent.height === "string"
                              ? selectedComponent.height
                              : `${selectedComponent.height}px`,
                          transform: `scale(${zoom[0] / 100})`,
                          transformOrigin: "top left",
                        }}
                      >
                        {/* Resize Handles */}
                        <div className="absolute -top-1 -left-1 w-2 h-2 bg-blue-500 rounded-full"></div>
                        <div className="absolute -top-1 -right-1 w-2 h-2 bg-blue-500 rounded-full"></div>
                        <div className="absolute -bottom-1 -left-1 w-2 h-2 bg-blue-500 rounded-full"></div>
                        <div className="absolute -bottom-1 -right-1 w-2 h-2 bg-blue-500 rounded-full"></div>
                      </div>
                    )}
                  </div>
                </div>

                {/* Canvas Controls */}
                <div className="flex items-center justify-between mt-4">
                  <div className="flex items-center gap-2">
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => setIsPlaying(!isPlaying)}
                      className="border-slate-600"
                    >
                      {isPlaying ? <Pause className="w-4 h-4" /> : <Play className="w-4 h-4" />}
                      {isPlaying ? "Pause" : "Preview"}
                    </Button>
                    <Button variant="outline" size="sm" className="border-slate-600">
                      <RotateCcw className="w-4 h-4 mr-2" />
                      Reset
                    </Button>
                  </div>

                  <div className="flex items-center gap-2">
                    <Button variant="outline" size="sm" className="border-slate-600">
                      <Save className="w-4 h-4 mr-2" />
                      Save
                    </Button>
                    <Button variant="outline" size="sm" onClick={exportPrototype} className="border-slate-600">
                      <Download className="w-4 h-4 mr-2" />
                      Export
                    </Button>
                    <Button variant="outline" size="sm" className="border-slate-600">
                      <Share className="w-4 h-4 mr-2" />
                      Share
                    </Button>
                  </div>
                </div>
              </CardContent>
            </Card>
          </div>

          {/* Right Sidebar - Properties & Code */}
          <div className="lg:col-span-1 space-y-6">
            <Tabs defaultValue="properties" className="w-full">
              <TabsList className="grid w-full grid-cols-2 bg-slate-800/50">
                <TabsTrigger value="properties" className="data-[state=active]:bg-slate-700">
                  Properties
                </TabsTrigger>
                <TabsTrigger value="code" className="data-[state=active]:bg-slate-700">
                  Code
                </TabsTrigger>
              </TabsList>

              <TabsContent value="properties" className="mt-4">
                <Card className="bg-slate-800/50 border-slate-700">
                  <CardHeader>
                    <CardTitle className="text-white text-sm">Component Properties</CardTitle>
                  </CardHeader>
                  <CardContent>
                    {selectedComponent ? (
                      <div className="space-y-4">
                        <div>
                          <label className="text-white text-sm font-medium mb-2 block">Type</label>
                          <Input
                            value={selectedComponent.type}
                            readOnly
                            className="bg-slate-700 border-slate-600 text-white"
                          />
                        </div>

                        <div className="grid grid-cols-2 gap-2">
                          <div>
                            <label className="text-white text-sm font-medium mb-2 block">X</label>
                            <Input
                              type="number"
                              value={selectedComponent.x}
                              onChange={(e) => {
                                setSelectedComponent((prev) => ({ ...prev, x: Number(e.target.value) }))
                                setCurrentPrototype((prev) => ({
                                  ...prev,
                                  components: prev.components.map((c) =>
                                    c.id === selectedComponent.id ? { ...c, x: Number(e.target.value) } : c,
                                  ),
                                }))
                              }}
                              className="bg-slate-700 border-slate-600 text-white"
                            />
                          </div>
                          <div>
                            <label className="text-white text-sm font-medium mb-2 block">Y</label>
                            <Input
                              type="number"
                              value={selectedComponent.y}
                              onChange={(e) => {
                                setSelectedComponent((prev) => ({ ...prev, y: Number(e.target.value) }))
                                setCurrentPrototype((prev) => ({
                                  ...prev,
                                  components: prev.components.map((c) =>
                                    c.id === selectedComponent.id ? { ...c, y: Number(e.target.value) } : c,
                                  ),
                                }))
                              }}
                              className="bg-slate-700 border-slate-600 text-white"
                            />
                          </div>
                        </div>

                        <div className="grid grid-cols-2 gap-2">
                          <div>
                            <label className="text-white text-sm font-medium mb-2 block">Width</label>
                            <Input
                              type="number"
                              value={selectedComponent.width}
                              onChange={(e) => {
                                setSelectedComponent((prev) => ({ ...prev, width: Number(e.target.value) }))
                                setCurrentPrototype((prev) => ({
                                  ...prev,
                                  components: prev.components.map((c) =>
                                    c.id === selectedComponent.id ? { ...c, width: Number(e.target.value) } : c,
                                  ),
                                }))
                              }}
                              className="bg-slate-700 border-slate-600 text-white"
                            />
                          </div>
                          <div>
                            <label className="text-white text-sm font-medium mb-2 block">Height</label>
                            <Input
                              type="number"
                              value={selectedComponent.height}
                              onChange={(e) => {
                                setSelectedComponent((prev) => ({ ...prev, height: Number(e.target.value) }))
                                setCurrentPrototype((prev) => ({
                                  ...prev,
                                  components: prev.components.map((c) =>
                                    c.id === selectedComponent.id ? { ...c, height: Number(e.target.value) } : c,
                                  ),
                                }))
                              }}
                              className="bg-slate-700 border-slate-600 text-white"
                            />
                          </div>
                        </div>

                        {/* Style Properties */}
                        <div>
                          <label className="text-white text-sm font-medium mb-2 block">Background Color</label>
                          <div className="flex gap-2">
                            <Input
                              type="color"
                              className="w-12 h-8 bg-slate-700 border-slate-600"
                              defaultValue="#ffffff"
                            />
                            <Input placeholder="#ffffff" className="flex-1 bg-slate-700 border-slate-600 text-white" />
                          </div>
                        </div>

                        <div>
                          <label className="text-white text-sm font-medium mb-2 block">Border Radius</label>
                          <Slider defaultValue={[0]} max={20} min={0} step={1} className="w-full" />
                        </div>

                        <div>
                          <label className="text-white text-sm font-medium mb-2 block">Opacity</label>
                          <Slider defaultValue={[100]} max={100} min={0} step={5} className="w-full" />
                        </div>
                      </div>
                    ) : (
                      <div className="text-center py-8">
                        <MousePointer className="w-12 h-12 text-gray-500 mx-auto mb-4" />
                        <p className="text-gray-400">Select a component to edit properties</p>
                      </div>
                    )}
                  </CardContent>
                </Card>
              </TabsContent>

              <TabsContent value="code" className="mt-4">
                <Card className="bg-slate-800/50 border-slate-700">
                  <CardHeader>
                    <div className="flex items-center justify-between">
                      <CardTitle className="text-white text-sm">Generated Code</CardTitle>
                      <Button variant="outline" size="sm" className="border-slate-600">
                        <Copy className="w-4 h-4" />
                      </Button>
                    </div>
                  </CardHeader>
                  <CardContent>
                    {selectedComponent ? (
                      <div className="space-y-4">
                        <div>
                          <label className="text-white text-sm font-medium mb-2 block">React Component</label>
                          <Textarea
                            value={generateComponentCode(selectedComponent)}
                            readOnly
                            className="min-h-32 bg-slate-900 border-slate-600 text-white font-mono text-xs"
                          />
                        </div>

                        <div>
                          <label className="text-white text-sm font-medium mb-2 block">CSS Styles</label>
                          <Textarea
                            value={generateComponentCSS(selectedComponent)}
                            readOnly
                            className="min-h-24 bg-slate-900 border-slate-600 text-white font-mono text-xs"
                          />
                        </div>
                      </div>
                    ) : (
                      <div className="text-center py-8">
                        <Code2 className="w-12 h-12 text-gray-500 mx-auto mb-4" />
                        <p className="text-gray-400">Select a component to view code</p>
                      </div>
                    )}
                  </CardContent>
                </Card>
              </TabsContent>
            </Tabs>

            {/* Layers Panel */}
            <Card className="bg-slate-800/50 border-slate-700">
              <CardHeader>
                <CardTitle className="text-white text-sm flex items-center gap-2">
                  <Layers className="w-4 h-4" />
                  Layers
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-1">
                  {currentPrototype.components.map((component, index) => (
                    <div
                      key={component.id}
                      className={`flex items-center gap-2 p-2 rounded cursor-pointer ${
                        selectedComponent?.id === component.id ? "bg-slate-700" : "hover:bg-slate-700/50"
                      }`}
                      onClick={() => setSelectedComponent(component)}
                    >
                      <div className="w-4 h-4 bg-gradient-to-r from-blue-500 to-purple-500 rounded"></div>
                      <span className="text-white text-sm flex-1">{component.type}</span>
                      <Eye className="w-4 h-4 text-gray-400" />
                    </div>
                  ))}
                  {currentPrototype.components.length === 0 && (
                    <div className="text-center py-4">
                      <p className="text-gray-400 text-sm">No components yet</p>
                      <p className="text-gray-500 text-xs">Drag components from the library</p>
                    </div>
                  )}
                </div>
              </CardContent>
            </Card>
          </div>
        </div>
      </div>
    </div>
  )
}

// Component Renderer
function ComponentRenderer({ component }) {
  const componentMap = {
    navbar: (
      <nav className="flex items-center justify-between p-4 bg-white shadow-sm border-b w-full h-full">
        <div className="text-lg font-bold">Logo</div>
        <div className="flex space-x-4 text-sm">
          <a href="#" className="text-gray-600 hover:text-gray-900">
            Home
          </a>
          <a href="#" className="text-gray-600 hover:text-gray-900">
            About
          </a>
          <a href="#" className="text-gray-600 hover:text-gray-900">
            Contact
          </a>
        </div>
      </nav>
    ),
    hero: (
      <section className="text-center py-12 bg-gradient-to-r from-blue-500 to-purple-600 text-white w-full h-full flex flex-col justify-center">
        <h1 className="text-2xl font-bold mb-2">Welcome to Our App</h1>
        <p className="text-sm mb-4">Build amazing things with our platform</p>
        <button className="px-4 py-2 bg-white text-blue-600 rounded font-semibold text-sm mx-auto">Get Started</button>
      </section>
    ),
    button: (
      <button className="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 w-full h-full text-sm">
        Click me
      </button>
    ),
    card: (
      <div className="p-4 bg-white rounded-lg shadow-md border w-full h-full">
        <h3 className="text-sm font-semibold mb-1">Card Title</h3>
        <p className="text-gray-600 text-xs">Card content goes here...</p>
      </div>
    ),
    input: (
      <input
        type="text"
        placeholder="Enter text..."
        className="w-full h-full px-3 py-2 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500 text-sm"
      />
    ),
  }

  return (
    componentMap[component.type] || (
      <div className="w-full h-full bg-gray-200 border-2 border-dashed border-gray-400 flex items-center justify-center">
        <span className="text-gray-500 text-xs">{component.type}</span>
      </div>
    )
  )
}

// Code Generation Functions
function generateComponentCode(component) {
  const codeMap = {
    navbar: `<nav className="flex items-center justify-between p-4 bg-white shadow-sm border-b">
  <div className="text-lg font-bold">Logo</div>
  <div className="flex space-x-4">
    <a href="#" className="text-gray-600 hover:text-gray-900">Home</a>
    <a href="#" className="text-gray-600 hover:text-gray-900">About</a>
    <a href="#" className="text-gray-600 hover:text-gray-900">Contact</a>
  </div>
</nav>`,
    hero: `<section className="text-center py-12 bg-gradient-to-r from-blue-500 to-purple-600 text-white">
  <h1 className="text-2xl font-bold mb-2">Welcome to Our App</h1>
  <p className="text-sm mb-4">Build amazing things with our platform</p>
  <button className="px-4 py-2 bg-white text-blue-600 rounded font-semibold">
    Get Started
  </button>
</section>`,
    button: `<button className="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600">
  Click me
</button>`,
  }

  return (
    codeMap[component.type] ||
    `<div className="component-${component.type}">
  {/* ${component.type} component */}
</div>`
  )
}

function generateComponentCSS(component) {
  return `.component-${component.type} {
  position: absolute;
  left: ${component.x}px;
  top: ${component.y}px;
  width: ${component.width}px;
  height: ${component.height}px;
}`
}
