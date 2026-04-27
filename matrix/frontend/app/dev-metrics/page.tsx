"use client"

import { useState, useEffect } from "react"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import {
  Activity,
  TrendingUp,
  Clock,
  Code2,
  GitCommit,
  Bug,
  ArrowLeft,
  Target,
  Award,
  BarChart3,
  PieChart,
  LineChart,
  Users,
} from "lucide-react"
import Link from "next/link"

export default function DevMetrics() {
  const [timeRange, setTimeRange] = useState("week")
  const [metrics, setMetrics] = useState(null)

  useEffect(() => {
    // Simulate loading metrics
    const timer = setTimeout(() => {
      setMetrics({
        productivity: {
          score: 87,
          trend: "+12%",
          linesOfCode: 2340,
          commits: 23,
          pullRequests: 8,
          codeReviews: 15,
        },
        quality: {
          score: 92,
          trend: "+5%",
          bugRate: 0.8,
          testCoverage: 84,
          codeComplexity: 6.2,
          duplicateCode: 3.1,
        },
        time: {
          totalHours: 42.5,
          codingTime: 28.3,
          debuggingTime: 8.7,
          meetingTime: 5.5,
          avgSessionLength: 2.1,
        },
        languages: [
          { name: "TypeScript", percentage: 45, hours: 19.1 },
          { name: "JavaScript", percentage: 25, hours: 10.6 },
          { name: "Python", percentage: 20, hours: 8.5 },
          { name: "CSS", percentage: 10, hours: 4.3 },
        ],
        dailyActivity: [
          { day: "Mon", commits: 4, hours: 8.2 },
          { day: "Tue", commits: 6, hours: 7.8 },
          { day: "Wed", commits: 3, hours: 6.5 },
          { day: "Thu", commits: 5, hours: 8.9 },
          { day: "Fri", commits: 5, hours: 7.1 },
          { day: "Sat", commits: 0, hours: 2.0 },
          { day: "Sun", commits: 0, hours: 2.0 },
        ],
      })
    }, 1000)

    return () => clearTimeout(timer)
  }, [timeRange])

  if (!metrics) {
    return (
      <div className="min-h-screen bg-gradient-to-br from-slate-900 via-red-900 to-slate-900 flex items-center justify-center">
        <div className="text-center">
          <Activity className="w-12 h-12 text-white mx-auto mb-4 animate-pulse" />
          <p className="text-white">Loading development metrics...</p>
        </div>
      </div>
    )
  }

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-900 via-red-900 to-slate-900">
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
            <div className="p-2 bg-red-500 rounded-lg">
              <Activity className="w-6 h-6 text-white" />
            </div>
            <div>
              <h1 className="text-3xl font-bold text-white">Development Metrics</h1>
              <p className="text-gray-300">Track productivity, code quality, and development patterns</p>
            </div>
          </div>
          <div className="ml-auto">
            <Select value={timeRange} onValueChange={setTimeRange}>
              <SelectTrigger className="w-32 bg-slate-800 border-slate-600">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="day">Today</SelectItem>
                <SelectItem value="week">This Week</SelectItem>
                <SelectItem value="month">This Month</SelectItem>
                <SelectItem value="quarter">This Quarter</SelectItem>
              </SelectContent>
            </Select>
          </div>
        </div>

        {/* Overview Cards */}
        <div className="grid grid-cols-2 md:grid-cols-4 gap-6 mb-8">
          <Card className="bg-slate-800/50 border-slate-700">
            <CardContent className="p-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-gray-400 text-sm">Productivity Score</p>
                  <p className="text-3xl font-bold text-white">{metrics.productivity.score}</p>
                  <p className="text-green-400 text-sm">{metrics.productivity.trend} from last week</p>
                </div>
                <TrendingUp className="w-8 h-8 text-green-500" />
              </div>
            </CardContent>
          </Card>

          <Card className="bg-slate-800/50 border-slate-700">
            <CardContent className="p-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-gray-400 text-sm">Code Quality</p>
                  <p className="text-3xl font-bold text-white">{metrics.quality.score}</p>
                  <p className="text-green-400 text-sm">{metrics.quality.trend} from last week</p>
                </div>
                <Award className="w-8 h-8 text-blue-500" />
              </div>
            </CardContent>
          </Card>

          <Card className="bg-slate-800/50 border-slate-700">
            <CardContent className="p-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-gray-400 text-sm">Total Hours</p>
                  <p className="text-3xl font-bold text-white">{metrics.time.totalHours}</p>
                  <p className="text-gray-400 text-sm">This week</p>
                </div>
                <Clock className="w-8 h-8 text-purple-500" />
              </div>
            </CardContent>
          </Card>

          <Card className="bg-slate-800/50 border-slate-700">
            <CardContent className="p-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-gray-400 text-sm">Commits</p>
                  <p className="text-3xl font-bold text-white">{metrics.productivity.commits}</p>
                  <p className="text-gray-400 text-sm">This week</p>
                </div>
                <GitCommit className="w-8 h-8 text-orange-500" />
              </div>
            </CardContent>
          </Card>
        </div>

        <div className="grid lg:grid-cols-3 gap-8">
          {/* Main Metrics */}
          <div className="lg:col-span-2">
            <Tabs defaultValue="productivity" className="w-full">
              <TabsList className="grid w-full grid-cols-4 bg-slate-800/50">
                <TabsTrigger value="productivity" className="data-[state=active]:bg-slate-700">
                  Productivity
                </TabsTrigger>
                <TabsTrigger value="quality" className="data-[state=active]:bg-slate-700">
                  Quality
                </TabsTrigger>
                <TabsTrigger value="time" className="data-[state=active]:bg-slate-700">
                  Time
                </TabsTrigger>
                <TabsTrigger value="activity" className="data-[state=active]:bg-slate-700">
                  Activity
                </TabsTrigger>
              </TabsList>

              <TabsContent value="productivity" className="mt-6">
                <Card className="bg-slate-800/50 border-slate-700">
                  <CardHeader>
                    <CardTitle className="text-white flex items-center gap-2">
                      <TrendingUp className="w-5 h-5" />
                      Productivity Metrics
                    </CardTitle>
                    <CardDescription className="text-gray-300">
                      Track your development output and efficiency
                    </CardDescription>
                  </CardHeader>
                  <CardContent className="space-y-6">
                    {/* Productivity Score */}
                    <div>
                      <div className="flex items-center justify-between mb-2">
                        <span className="text-white font-medium">Overall Productivity Score</span>
                        <Badge variant="secondary" className="text-lg px-3 py-1">
                          {metrics.productivity.score}/100
                        </Badge>
                      </div>
                      <div className="h-3 bg-slate-700 rounded-full overflow-hidden">
                        <div
                          className="h-full bg-gradient-to-r from-green-500 to-blue-500"
                          style={{ width: `${metrics.productivity.score}%` }}
                        ></div>
                      </div>
                    </div>

                    {/* Key Metrics */}
                    <div className="grid grid-cols-2 gap-4">
                      <div className="p-4 bg-slate-700/30 rounded-lg">
                        <div className="flex items-center gap-2 mb-2">
                          <Code2 className="w-4 h-4 text-blue-400" />
                          <span className="text-white font-medium">Lines of Code</span>
                        </div>
                        <div className="text-2xl font-bold text-white">
                          {metrics.productivity.linesOfCode.toLocaleString()}
                        </div>
                        <div className="text-sm text-gray-400">This week</div>
                      </div>

                      <div className="p-4 bg-slate-700/30 rounded-lg">
                        <div className="flex items-center gap-2 mb-2">
                          <GitCommit className="w-4 h-4 text-green-400" />
                          <span className="text-white font-medium">Pull Requests</span>
                        </div>
                        <div className="text-2xl font-bold text-white">{metrics.productivity.pullRequests}</div>
                        <div className="text-sm text-gray-400">This week</div>
                      </div>

                      <div className="p-4 bg-slate-700/30 rounded-lg">
                        <div className="flex items-center gap-2 mb-2">
                          <Target className="w-4 h-4 text-purple-400" />
                          <span className="text-white font-medium">Code Reviews</span>
                        </div>
                        <div className="text-2xl font-bold text-white">{metrics.productivity.codeReviews}</div>
                        <div className="text-sm text-gray-400">This week</div>
                      </div>

                      <div className="p-4 bg-slate-700/30 rounded-lg">
                        <div className="flex items-center gap-2 mb-2">
                          <GitCommit className="w-4 h-4 text-orange-400" />
                          <span className="text-white font-medium">Commits</span>
                        </div>
                        <div className="text-2xl font-bold text-white">{metrics.productivity.commits}</div>
                        <div className="text-sm text-gray-400">This week</div>
                      </div>
                    </div>
                  </CardContent>
                </Card>
              </TabsContent>

              <TabsContent value="quality" className="mt-6">
                <Card className="bg-slate-800/50 border-slate-700">
                  <CardHeader>
                    <CardTitle className="text-white flex items-center gap-2">
                      <Award className="w-5 h-5" />
                      Code Quality Metrics
                    </CardTitle>
                    <CardDescription className="text-gray-300">
                      Monitor code quality and maintainability
                    </CardDescription>
                  </CardHeader>
                  <CardContent className="space-y-6">
                    {/* Quality Score */}
                    <div>
                      <div className="flex items-center justify-between mb-2">
                        <span className="text-white font-medium">Code Quality Score</span>
                        <Badge variant="secondary" className="text-lg px-3 py-1">
                          {metrics.quality.score}/100
                        </Badge>
                      </div>
                      <div className="h-3 bg-slate-700 rounded-full overflow-hidden">
                        <div
                          className="h-full bg-gradient-to-r from-blue-500 to-purple-500"
                          style={{ width: `${metrics.quality.score}%` }}
                        ></div>
                      </div>
                    </div>

                    {/* Quality Metrics */}
                    <div className="grid grid-cols-2 gap-4">
                      <div className="p-4 bg-slate-700/30 rounded-lg">
                        <div className="flex items-center gap-2 mb-2">
                          <Bug className="w-4 h-4 text-red-400" />
                          <span className="text-white font-medium">Bug Rate</span>
                        </div>
                        <div className="text-2xl font-bold text-white">{metrics.quality.bugRate}%</div>
                        <div className="text-sm text-green-400">↓ 0.3% from last week</div>
                      </div>

                      <div className="p-4 bg-slate-700/30 rounded-lg">
                        <div className="flex items-center gap-2 mb-2">
                          <Target className="w-4 h-4 text-green-400" />
                          <span className="text-white font-medium">Test Coverage</span>
                        </div>
                        <div className="text-2xl font-bold text-white">{metrics.quality.testCoverage}%</div>
                        <div className="text-sm text-green-400">↑ 2% from last week</div>
                      </div>

                      <div className="p-4 bg-slate-700/30 rounded-lg">
                        <div className="flex items-center gap-2 mb-2">
                          <BarChart3 className="w-4 h-4 text-yellow-400" />
                          <span className="text-white font-medium">Complexity</span>
                        </div>
                        <div className="text-2xl font-bold text-white">{metrics.quality.codeComplexity}</div>
                        <div className="text-sm text-gray-400">Cyclomatic complexity</div>
                      </div>

                      <div className="p-4 bg-slate-700/30 rounded-lg">
                        <div className="flex items-center gap-2 mb-2">
                          <PieChart className="w-4 h-4 text-blue-400" />
                          <span className="text-white font-medium">Duplicate Code</span>
                        </div>
                        <div className="text-2xl font-bold text-white">{metrics.quality.duplicateCode}%</div>
                        <div className="text-sm text-green-400">↓ 0.5% from last week</div>
                      </div>
                    </div>
                  </CardContent>
                </Card>
              </TabsContent>

              <TabsContent value="time" className="mt-6">
                <Card className="bg-slate-800/50 border-slate-700">
                  <CardHeader>
                    <CardTitle className="text-white flex items-center gap-2">
                      <Clock className="w-5 h-5" />
                      Time Analysis
                    </CardTitle>
                    <CardDescription className="text-gray-300">
                      Understand how you spend your development time
                    </CardDescription>
                  </CardHeader>
                  <CardContent className="space-y-6">
                    {/* Time Breakdown */}
                    <div className="grid grid-cols-2 gap-4">
                      <div className="p-4 bg-slate-700/30 rounded-lg">
                        <div className="flex items-center gap-2 mb-2">
                          <Code2 className="w-4 h-4 text-green-400" />
                          <span className="text-white font-medium">Coding Time</span>
                        </div>
                        <div className="text-2xl font-bold text-white">{metrics.time.codingTime}h</div>
                        <div className="text-sm text-gray-400">66% of total time</div>
                      </div>

                      <div className="p-4 bg-slate-700/30 rounded-lg">
                        <div className="flex items-center gap-2 mb-2">
                          <Bug className="w-4 h-4 text-red-400" />
                          <span className="text-white font-medium">Debugging</span>
                        </div>
                        <div className="text-2xl font-bold text-white">{metrics.time.debuggingTime}h</div>
                        <div className="text-sm text-gray-400">20% of total time</div>
                      </div>

                      <div className="p-4 bg-slate-700/30 rounded-lg">
                        <div className="flex items-center gap-2 mb-2">
                          <Users className="w-4 h-4 text-blue-400" />
                          <span className="text-white font-medium">Meetings</span>
                        </div>
                        <div className="text-2xl font-bold text-white">{metrics.time.meetingTime}h</div>
                        <div className="text-sm text-gray-400">13% of total time</div>
                      </div>

                      <div className="p-4 bg-slate-700/30 rounded-lg">
                        <div className="flex items-center gap-2 mb-2">
                          <Clock className="w-4 h-4 text-purple-400" />
                          <span className="text-white font-medium">Avg Session</span>
                        </div>
                        <div className="text-2xl font-bold text-white">{metrics.time.avgSessionLength}h</div>
                        <div className="text-sm text-gray-400">Per coding session</div>
                      </div>
                    </div>

                    {/* Time Distribution Chart */}
                    <div>
                      <h4 className="text-white font-medium mb-3">Time Distribution</h4>
                      <div className="space-y-2">
                        <div className="flex items-center justify-between">
                          <span className="text-gray-300">Coding</span>
                          <span className="text-white">{metrics.time.codingTime}h</span>
                        </div>
                        <div className="h-2 bg-slate-700 rounded-full overflow-hidden">
                          <div className="h-full bg-green-500" style={{ width: "66%" }}></div>
                        </div>

                        <div className="flex items-center justify-between">
                          <span className="text-gray-300">Debugging</span>
                          <span className="text-white">{metrics.time.debuggingTime}h</span>
                        </div>
                        <div className="h-2 bg-slate-700 rounded-full overflow-hidden">
                          <div className="h-full bg-red-500" style={{ width: "20%" }}></div>
                        </div>

                        <div className="flex items-center justify-between">
                          <span className="text-gray-300">Meetings</span>
                          <span className="text-white">{metrics.time.meetingTime}h</span>
                        </div>
                        <div className="h-2 bg-slate-700 rounded-full overflow-hidden">
                          <div className="h-full bg-blue-500" style={{ width: "13%" }}></div>
                        </div>
                      </div>
                    </div>
                  </CardContent>
                </Card>
              </TabsContent>

              <TabsContent value="activity" className="mt-6">
                <Card className="bg-slate-800/50 border-slate-700">
                  <CardHeader>
                    <CardTitle className="text-white flex items-center gap-2">
                      <LineChart className="w-5 h-5" />
                      Daily Activity
                    </CardTitle>
                    <CardDescription className="text-gray-300">
                      Your development activity throughout the week
                    </CardDescription>
                  </CardHeader>
                  <CardContent>
                    <div className="space-y-4">
                      {metrics.dailyActivity.map((day) => (
                        <div key={day.day} className="flex items-center gap-4">
                          <div className="w-12 text-gray-300 text-sm">{day.day}</div>
                          <div className="flex-1">
                            <div className="flex items-center justify-between mb-1">
                              <span className="text-white text-sm">{day.hours}h coding</span>
                              <span className="text-gray-400 text-sm">{day.commits} commits</span>
                            </div>
                            <div className="h-2 bg-slate-700 rounded-full overflow-hidden">
                              <div
                                className="h-full bg-gradient-to-r from-blue-500 to-purple-500"
                                style={{ width: `${(day.hours / 10) * 100}%` }}
                              ></div>
                            </div>
                          </div>
                        </div>
                      ))}
                    </div>
                  </CardContent>
                </Card>
              </TabsContent>
            </Tabs>
          </div>

          {/* Sidebar */}
          <div className="space-y-6">
            {/* Language Breakdown */}
            <Card className="bg-slate-800/50 border-slate-700">
              <CardHeader>
                <CardTitle className="text-white flex items-center gap-2">
                  <Code2 className="w-5 h-5" />
                  Languages
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-3">
                  {metrics.languages.map((lang) => (
                    <div key={lang.name}>
                      <div className="flex items-center justify-between mb-1">
                        <span className="text-white text-sm">{lang.name}</span>
                        <span className="text-gray-400 text-sm">{lang.hours}h</span>
                      </div>
                      <div className="h-2 bg-slate-700 rounded-full overflow-hidden">
                        <div
                          className="h-full bg-gradient-to-r from-blue-500 to-purple-500"
                          style={{ width: `${lang.percentage}%` }}
                        ></div>
                      </div>
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>

            {/* Goals */}
            <Card className="bg-slate-800/50 border-slate-700">
              <CardHeader>
                <CardTitle className="text-white flex items-center gap-2">
                  <Target className="w-5 h-5" />
                  Weekly Goals
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-3">
                  <div>
                    <div className="flex items-center justify-between mb-1">
                      <span className="text-white text-sm">Commits</span>
                      <span className="text-gray-400 text-sm">{metrics.productivity.commits}/25</span>
                    </div>
                    <div className="h-2 bg-slate-700 rounded-full overflow-hidden">
                      <div
                        className="h-full bg-green-500"
                        style={{ width: `${(metrics.productivity.commits / 25) * 100}%` }}
                      ></div>
                    </div>
                  </div>

                  <div>
                    <div className="flex items-center justify-between mb-1">
                      <span className="text-white text-sm">Code Reviews</span>
                      <span className="text-gray-400 text-sm">{metrics.productivity.codeReviews}/20</span>
                    </div>
                    <div className="h-2 bg-slate-700 rounded-full overflow-hidden">
                      <div
                        className="h-full bg-blue-500"
                        style={{ width: `${(metrics.productivity.codeReviews / 20) * 100}%` }}
                      ></div>
                    </div>
                  </div>

                  <div>
                    <div className="flex items-center justify-between mb-1">
                      <span className="text-white text-sm">Coding Hours</span>
                      <span className="text-gray-400 text-sm">{metrics.time.codingTime}/40</span>
                    </div>
                    <div className="h-2 bg-slate-700 rounded-full overflow-hidden">
                      <div
                        className="h-full bg-purple-500"
                        style={{ width: `${(metrics.time.codingTime / 40) * 100}%` }}
                      ></div>
                    </div>
                  </div>
                </div>
              </CardContent>
            </Card>

            {/* Achievements */}
            <Card className="bg-slate-800/50 border-slate-700">
              <CardHeader>
                <CardTitle className="text-white flex items-center gap-2">
                  <Award className="w-5 h-5" />
                  Recent Achievements
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-3">
                  <div className="flex items-center gap-3 p-2 bg-slate-700/30 rounded">
                    <Award className="w-5 h-5 text-yellow-500" />
                    <div>
                      <div className="text-white text-sm font-medium">Code Quality Master</div>
                      <div className="text-gray-400 text-xs">Maintained 90%+ quality score</div>
                    </div>
                  </div>
                  <div className="flex items-center gap-3 p-2 bg-slate-700/30 rounded">
                    <GitCommit className="w-5 h-5 text-green-500" />
                    <div>
                      <div className="text-white text-sm font-medium">Commit Streak</div>
                      <div className="text-gray-400 text-xs">7 days of consistent commits</div>
                    </div>
                  </div>
                  <div className="flex items-center gap-3 p-2 bg-slate-700/30 rounded">
                    <Clock className="w-5 h-5 text-blue-500" />
                    <div>
                      <div className="text-white text-sm font-medium">Productivity Boost</div>
                      <div className="text-gray-400 text-xs">20% increase this week</div>
                    </div>
                  </div>
                </div>
              </CardContent>
            </Card>
          </div>
        </div>
      </div>
    </div>
  )
}
