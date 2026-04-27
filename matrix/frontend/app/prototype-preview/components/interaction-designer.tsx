"use client"

import { useState } from "react"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Badge } from "@/components/ui/badge"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Slider } from "@/components/ui/slider"
import { Play, Pause, Settings, Plus, Trash2, Copy, Eye, EyeOff, Sparkles } from "lucide-react"

export function InteractionDesigner() {
  const [selectedInteraction, setSelectedInteraction] = useState(null)
  const [isPlaying, setIsPlaying] = useState(false)
  const [interactions, setInteractions] = useState([
    {
      id: "hover-1",
      name: "Button Hover Effect",
      trigger: "hover",
      target: "button",
      animation: "scale",
      duration: 200,
      easing: "ease-out",
      properties: { scale: 1.05, opacity: 0.9 },
      enabled: true,
    },
    {
      id: "click-1",
      name: "Card Click Animation",
      trigger: "click",
      target: "card",
      animation: "bounce",
      duration: 300,
      easing: "ease-in-out",
      properties: { scale: 0.95 },
      enabled: true,
    },
    {
      id: "scroll-1",
      name: "Fade In on Scroll",
      trigger: "scroll",
      target: "section",
      animation: "fadeIn",
      duration: 500,
      easing: "ease-out",
      properties: { opacity: 1, translateY: 0 },
      enabled: true,
    },
  ])

  const [transitions, setTransitions] = useState([
    {
      id: "page-1",
      name: "Page Slide Transition",
      type: "page",
      animation: "slideLeft",
      duration: 400,
      easing: "ease-in-out",
      enabled: true,
    },
    {
      id: "modal-1",
      name: "Modal Fade In",
      type: "modal",
      animation: "fadeIn",
      duration: 250,
      easing: "ease-out",
      enabled: true,
    },
  ])

  const animationTypes = [
    { id: "fadeIn", name: "Fade In", icon: "✨" },
    { id: "fadeOut", name: "Fade Out", icon: "💫" },
    { id: "slideUp", name: "Slide Up", icon: "⬆️" },
    { id: "slideDown", name: "Slide Down", icon: "⬇️" },
    { id: "slideLeft", name: "Slide Left", icon: "⬅️" },
    { id: "slideRight", name: "Slide Right", icon: "➡️" },
    { id: "scale", name: "Scale", icon: "🔍" },
    { id: "bounce", name: "Bounce", icon: "🏀" },
    { id: "shake", name: "Shake", icon: "📳" },
    { id: "rotate", name: "Rotate", icon: "🔄" },
    { id: "flip", name: "Flip", icon: "🔃" },
    { id: "pulse", name: "Pulse", icon: "💓" },
  ]

  const triggerTypes = [
    { id: "hover", name: "Hover", icon: "👆" },
    { id: "click", name: "Click", icon: "👆" },
    { id: "focus", name: "Focus", icon: "🎯" },
    { id: "scroll", name: "Scroll", icon: "📜" },
    { id: "load", name: "Page Load", icon: "⚡" },
    { id: "resize", name: "Resize", icon: "📏" },
  ]

  const easingTypes = [
    { id: "linear", name: "Linear" },
    { id: "ease", name: "Ease" },
    { id: "ease-in", name: "Ease In" },
    { id: "ease-out", name: "Ease Out" },
    { id: "ease-in-out", name: "Ease In Out" },
    { id: "cubic-bezier", name: "Custom Cubic Bezier" },
  ]

  const addInteraction = () => {
    const newInteraction = {
      id: `interaction-${Date.now()}`,
      name: "New Interaction",
      trigger: "hover",
      target: "element",
      animation: "fadeIn",
      duration: 300,
      easing: "ease-out",
      properties: {},
      enabled: true,
    }
    setInteractions([...interactions, newInteraction])
  }

  const duplicateInteraction = (interaction) => {
    const duplicated = {
      ...interaction,
      id: `${interaction.id}-copy-${Date.now()}`,
      name: `${interaction.name} Copy`,
    }
    setInteractions([...interactions, duplicated])
  }

  const deleteInteraction = (id) => {
    setInteractions(interactions.filter((i) => i.id !== id))
    if (selectedInteraction?.id === id) {
      setSelectedInteraction(null)
    }
  }

  const toggleInteraction = (id) => {
    setInteractions(interactions.map((i) => (i.id === id ? { ...i, enabled: !i.enabled } : i)))
  }

  const previewInteraction = (interaction) => {
    setIsPlaying(true)
    // Simulate animation preview
    setTimeout(() => {
      setIsPlaying(false)
    }, interaction.duration)
  }

  const generateCSS = (interaction) => {
    const keyframeName = `${interaction.animation}-${interaction.id}`

    let keyframes = ""
    let properties = ""

    switch (interaction.animation) {
      case "fadeIn":
        keyframes = `@keyframes ${keyframeName} {
  from { opacity: 0; }
  to { opacity: 1; }
}`
        properties = `animation: ${keyframeName} ${interaction.duration}ms ${interaction.easing};`
        break
      case "slideUp":
        keyframes = `@keyframes ${keyframeName} {
  from { transform: translateY(20px); opacity: 0; }
  to { transform: translateY(0); opacity: 1; }
}`
        properties = `animation: ${keyframeName} ${interaction.duration}ms ${interaction.easing};`
        break
      case "scale":
        keyframes = `@keyframes ${keyframeName} {
  from { transform: scale(0.8); }
  to { transform: scale(1); }
}`
        properties = `animation: ${keyframeName} ${interaction.duration}ms ${interaction.easing};`
        break
      default:
        keyframes = `@keyframes ${keyframeName} {
  /* Custom animation */
}`
        properties = `animation: ${keyframeName} ${interaction.duration}ms ${interaction.easing};`
    }

    return { keyframes, properties }
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-white">Interaction Designer</h2>
          <p className="text-gray-300">Create engaging animations and micro-interactions</p>
        </div>
        <div className="flex gap-2">
          <Button onClick={() => setIsPlaying(!isPlaying)} variant="outline" className="border-slate-600">
            {isPlaying ? <Pause className="w-4 h-4 mr-2" /> : <Play className="w-4 h-4 mr-2" />}
            {isPlaying ? "Pause" : "Preview All"}
          </Button>
          <Button onClick={addInteraction} className="bg-indigo-600 hover:bg-indigo-700">
            <Plus className="w-4 h-4 mr-2" />
            Add Interaction
          </Button>
        </div>
      </div>

      <div className="grid lg:grid-cols-3 gap-6">
        {/* Interactions List */}
        <div className="lg:col-span-1">
          <Card className="bg-slate-800/50 border-slate-700">
            <CardHeader>
              <CardTitle className="text-white flex items-center gap-2">
                <Sparkles className="w-5 h-5" />
                Interactions
              </CardTitle>
              <CardDescription className="text-gray-300">Manage your component interactions</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="space-y-2">
                {interactions.map((interaction) => (
                  <div
                    key={interaction.id}
                    className={`p-3 rounded-lg cursor-pointer transition-colors ${
                      selectedInteraction?.id === interaction.id
                        ? "bg-slate-700"
                        : "bg-slate-700/30 hover:bg-slate-700/50"
                    }`}
                    onClick={() => setSelectedInteraction(interaction)}
                  >
                    <div className="flex items-center justify-between mb-2">
                      <div className="flex items-center gap-2">
                        <span className="text-lg">
                          {animationTypes.find((a) => a.id === interaction.animation)?.icon || "✨"}
                        </span>
                        <span className="text-white font-medium text-sm">{interaction.name}</span>
                      </div>
                      <div className="flex items-center gap-1">
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={(e) => {
                            e.stopPropagation()
                            toggleInteraction(interaction.id)
                          }}
                          className="p-1 h-auto"
                        >
                          {interaction.enabled ? (
                            <Eye className="w-3 h-3 text-green-400" />
                          ) : (
                            <EyeOff className="w-3 h-3 text-gray-500" />
                          )}
                        </Button>
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={(e) => {
                            e.stopPropagation()
                            previewInteraction(interaction)
                          }}
                          className="p-1 h-auto"
                        >
                          <Play className="w-3 h-3 text-blue-400" />
                        </Button>
                      </div>
                    </div>
                    <div className="flex items-center gap-2 text-xs text-gray-400">
                      <Badge variant="outline" className="text-xs border-slate-600">
                        {interaction.trigger}
                      </Badge>
                      <span>•</span>
                      <span>{interaction.duration}ms</span>
                      <span>•</span>
                      <span>{interaction.animation}</span>
                    </div>
                  </div>
                ))}
                {interactions.length === 0 && (
                  <div className="text-center py-8">
                    <Sparkles className="w-12 h-12 text-gray-500 mx-auto mb-4" />
                    <p className="text-gray-400">No interactions yet</p>
                    <p className="text-gray-500 text-sm">Create your first interaction</p>
                  </div>
                )}
              </div>
            </CardContent>
          </Card>
        </div>

        {/* Interaction Editor */}
        <div className="lg:col-span-2">
          <Tabs defaultValue="properties" className="w-full">
            <TabsList className="grid w-full grid-cols-3 bg-slate-800/50">
              <TabsTrigger value="properties" className="data-[state=active]:bg-slate-700">
                Properties
              </TabsTrigger>
              <TabsTrigger value="timeline" className="data-[state=active]:bg-slate-700">
                Timeline
              </TabsTrigger>
              <TabsTrigger value="code" className="data-[state=active]:bg-slate-700">
                Code
              </TabsTrigger>
            </TabsList>

            <TabsContent value="properties" className="mt-6">
              <Card className="bg-slate-800/50 border-slate-700">
                <CardHeader>
                  <div className="flex items-center justify-between">
                    <CardTitle className="text-white">Interaction Properties</CardTitle>
                    {selectedInteraction && (
                      <div className="flex gap-2">
                        <Button
                          variant="outline"
                          size="sm"
                          onClick={() => duplicateInteraction(selectedInteraction)}
                          className="border-slate-600"
                        >
                          <Copy className="w-4 h-4" />
                        </Button>
                        <Button
                          variant="outline"
                          size="sm"
                          onClick={() => deleteInteraction(selectedInteraction.id)}
                          className="border-slate-600 hover:border-red-500 hover:text-red-400"
                        >
                          <Trash2 className="w-4 h-4" />
                        </Button>
                      </div>
                    )}
                  </div>
                </CardHeader>
                <CardContent>
                  {selectedInteraction ? (
                    <div className="space-y-6">
                      {/* Basic Properties */}
                      <div className="grid grid-cols-2 gap-4">
                        <div>
                          <label className="text-white text-sm font-medium mb-2 block">Name</label>
                          <Input
                            value={selectedInteraction.name}
                            onChange={(e) => {
                              setSelectedInteraction({ ...selectedInteraction, name: e.target.value })
                              setInteractions(
                                interactions.map((i) =>
                                  i.id === selectedInteraction.id ? { ...i, name: e.target.value } : i,
                                ),
                              )
                            }}
                            className="bg-slate-700 border-slate-600 text-white"
                          />
                        </div>
                        <div>
                          <label className="text-white text-sm font-medium mb-2 block">Target Element</label>
                          <Input
                            value={selectedInteraction.target}
                            onChange={(e) => {
                              setSelectedInteraction({ ...selectedInteraction, target: e.target.value })
                              setInteractions(
                                interactions.map((i) =>
                                  i.id === selectedInteraction.id ? { ...i, target: e.target.value } : i,
                                ),
                              )
                            }}
                            placeholder="CSS selector or element ID"
                            className="bg-slate-700 border-slate-600 text-white"
                          />
                        </div>
                      </div>

                      {/* Trigger and Animation */}
                      <div className="grid grid-cols-2 gap-4">
                        <div>
                          <label className="text-white text-sm font-medium mb-2 block">Trigger</label>
                          <Select
                            value={selectedInteraction.trigger}
                            onValueChange={(value) => {
                              setSelectedInteraction({ ...selectedInteraction, trigger: value })
                              setInteractions(
                                interactions.map((i) =>
                                  i.id === selectedInteraction.id ? { ...i, trigger: value } : i,
                                ),
                              )
                            }}
                          >
                            <SelectTrigger className="bg-slate-700 border-slate-600">
                              <SelectValue />
                            </SelectTrigger>
                            <SelectContent>
                              {triggerTypes.map((trigger) => (
                                <SelectItem key={trigger.id} value={trigger.id}>
                                  <div className="flex items-center gap-2">
                                    <span>{trigger.icon}</span>
                                    <span>{trigger.name}</span>
                                  </div>
                                </SelectItem>
                              ))}
                            </SelectContent>
                          </Select>
                        </div>
                        <div>
                          <label className="text-white text-sm font-medium mb-2 block">Animation</label>
                          <Select
                            value={selectedInteraction.animation}
                            onValueChange={(value) => {
                              setSelectedInteraction({ ...selectedInteraction, animation: value })
                              setInteractions(
                                interactions.map((i) =>
                                  i.id === selectedInteraction.id ? { ...i, animation: value } : i,
                                ),
                              )
                            }}
                          >
                            <SelectTrigger className="bg-slate-700 border-slate-600">
                              <SelectValue />
                            </SelectTrigger>
                            <SelectContent>
                              {animationTypes.map((animation) => (
                                <SelectItem key={animation.id} value={animation.id}>
                                  <div className="flex items-center gap-2">
                                    <span>{animation.icon}</span>
                                    <span>{animation.name}</span>
                                  </div>
                                </SelectItem>
                              ))}
                            </SelectContent>
                          </Select>
                        </div>
                      </div>

                      {/* Timing */}
                      <div className="grid grid-cols-2 gap-4">
                        <div>
                          <label className="text-white text-sm font-medium mb-2 block">
                            Duration: {selectedInteraction.duration}ms
                          </label>
                          <Slider
                            value={[selectedInteraction.duration]}
                            onValueChange={([value]) => {
                              setSelectedInteraction({ ...selectedInteraction, duration: value })
                              setInteractions(
                                interactions.map((i) =>
                                  i.id === selectedInteraction.id ? { ...i, duration: value } : i,
                                ),
                              )
                            }}
                            max={2000}
                            min={50}
                            step={50}
                            className="w-full"
                          />
                        </div>
                        <div>
                          <label className="text-white text-sm font-medium mb-2 block">Easing</label>
                          <Select
                            value={selectedInteraction.easing}
                            onValueChange={(value) => {
                              setSelectedInteraction({ ...selectedInteraction, easing: value })
                              setInteractions(
                                interactions.map((i) =>
                                  i.id === selectedInteraction.id ? { ...i, easing: value } : i,
                                ),
                              )
                            }}
                          >
                            <SelectTrigger className="bg-slate-700 border-slate-600">
                              <SelectValue />
                            </SelectTrigger>
                            <SelectContent>
                              {easingTypes.map((easing) => (
                                <SelectItem key={easing.id} value={easing.id}>
                                  {easing.name}
                                </SelectItem>
                              ))}
                            </SelectContent>
                          </Select>
                        </div>
                      </div>

                      {/* Preview */}
                      <div className="p-4 bg-slate-700/30 rounded-lg">
                        <div className="flex items-center justify-between mb-3">
                          <h4 className="text-white font-medium">Preview</h4>
                          <Button
                            onClick={() => previewInteraction(selectedInteraction)}
                            variant="outline"
                            size="sm"
                            className="border-slate-600"
                          >
                            <Play className="w-4 h-4 mr-2" />
                            Test Animation
                          </Button>
                        </div>
                        <div className="h-32 bg-slate-600/30 rounded flex items-center justify-center">
                          <div
                            className={`w-16 h-16 bg-gradient-to-r from-blue-500 to-purple-500 rounded-lg transition-all ${
                              isPlaying ? "animate-pulse" : ""
                            }`}
                            style={{
                              animationDuration: `${selectedInteraction.duration}ms`,
                              animationTimingFunction: selectedInteraction.easing,
                            }}
                          ></div>
                        </div>
                      </div>
                    </div>
                  ) : (
                    <div className="text-center py-12">
                      <Settings className="w-12 h-12 text-gray-500 mx-auto mb-4" />
                      <p className="text-gray-400">Select an interaction to edit properties</p>
                    </div>
                  )}
                </CardContent>
              </Card>
            </TabsContent>

            <TabsContent value="timeline" className="mt-6">
              <Card className="bg-slate-800/50 border-slate-700">
                <CardHeader>
                  <CardTitle className="text-white">Animation Timeline</CardTitle>
                  <CardDescription className="text-gray-300">Visualize and fine-tune animation timing</CardDescription>
                </CardHeader>
                <CardContent>
                  <div className="space-y-4">
                    {/* Timeline Ruler */}
                    <div className="relative h-8 bg-slate-700/30 rounded">
                      <div className="absolute inset-0 flex items-center">
                        {Array.from({ length: 11 }, (_, i) => (
                          <div key={i} className="flex-1 text-center">
                            <div className="w-px h-4 bg-slate-500 mx-auto"></div>
                            <span className="text-xs text-gray-400">{i * 100}ms</span>
                          </div>
                        ))}
                      </div>
                    </div>

                    {/* Animation Tracks */}
                    {selectedInteraction && (
                      <div className="space-y-2">
                        <div className="flex items-center gap-4">
                          <div className="w-32 text-white text-sm">{selectedInteraction.name}</div>
                          <div className="flex-1 relative h-8 bg-slate-700/30 rounded">
                            <div
                              className="absolute top-1 bottom-1 bg-gradient-to-r from-blue-500 to-purple-500 rounded"
                              style={{
                                left: "0%",
                                width: `${(selectedInteraction.duration / 1000) * 100}%`,
                              }}
                            ></div>
                          </div>
                          <div className="w-20 text-right text-sm text-gray-400">{selectedInteraction.duration}ms</div>
                        </div>
                      </div>
                    )}

                    {/* Keyframe Editor */}
                    <div className="mt-6 p-4 bg-slate-700/30 rounded-lg">
                      <h4 className="text-white font-medium mb-3">Keyframes</h4>
                      <div className="space-y-2">
                        <div className="flex items-center gap-4 text-sm">
                          <span className="w-16 text-gray-400">0%</span>
                          <span className="text-white">Initial state</span>
                        </div>
                        <div className="flex items-center gap-4 text-sm">
                          <span className="w-16 text-gray-400">100%</span>
                          <span className="text-white">Final state</span>
                        </div>
                      </div>
                      <Button variant="outline" size="sm" className="mt-3 border-slate-600">
                        <Plus className="w-4 h-4 mr-2" />
                        Add Keyframe
                      </Button>
                    </div>
                  </div>
                </CardContent>
              </Card>
            </TabsContent>

            <TabsContent value="code" className="mt-6">
              <Card className="bg-slate-800/50 border-slate-700">
                <CardHeader>
                  <div className="flex items-center justify-between">
                    <CardTitle className="text-white">Generated Code</CardTitle>
                    <Button variant="outline" size="sm" className="border-slate-600">
                      <Copy className="w-4 h-4 mr-2" />
                      Copy CSS
                    </Button>
                  </div>
                </CardHeader>
                <CardContent>
                  {selectedInteraction ? (
                    <div className="space-y-4">
                      <div>
                        <label className="text-white text-sm font-medium mb-2 block">CSS Animation</label>
                        <pre className="bg-slate-900 border border-slate-600 rounded-lg p-4 text-sm text-white overflow-auto">
                          <code>{generateCSS(selectedInteraction).keyframes}</code>
                        </pre>
                      </div>
                      <div>
                        <label className="text-white text-sm font-medium mb-2 block">CSS Class</label>
                        <pre className="bg-slate-900 border border-slate-600 rounded-lg p-4 text-sm text-white overflow-auto">
                          <code>{`.${selectedInteraction.target}:${selectedInteraction.trigger} {
  ${generateCSS(selectedInteraction).properties}
}`}</code>
                        </pre>
                      </div>
                      <div>
                        <label className="text-white text-sm font-medium mb-2 block">JavaScript (Optional)</label>
                        <pre className="bg-slate-900 border border-slate-600 rounded-lg p-4 text-sm text-white overflow-auto">
                          <code>{`document.querySelector('${selectedInteraction.target}').addEventListener('${selectedInteraction.trigger}', function() {
  this.style.animation = '${selectedInteraction.animation}-${selectedInteraction.id} ${selectedInteraction.duration}ms ${selectedInteraction.easing}';
});`}</code>
                        </pre>
                      </div>
                    </div>
                  ) : (
                    <div className="text-center py-12">
                      <Copy className="w-12 h-12 text-gray-500 mx-auto mb-4" />
                      <p className="text-gray-400">Select an interaction to view generated code</p>
                    </div>
                  )}
                </CardContent>
              </Card>
            </TabsContent>
          </Tabs>
        </div>
      </div>
    </div>
  )
}
