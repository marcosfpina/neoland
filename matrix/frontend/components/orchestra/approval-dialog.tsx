/**
 * Approval Dialog Component
 * ==========================
 * Dialog for human review of agent decisions requiring manual approval.
 *
 * Triggered when:
 * - Decision score < 0.90
 * - STF violations detected
 * - Critical decision types (deploy, delete_db, modify_acl)
 */

"use client"

import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog"
import { Badge } from "@/components/ui/badge"
import { Card, CardContent } from "@/components/ui/card"
import { AlertCircle, CheckCircle, XCircle, AlertTriangle } from "lucide-react"
import type { RankingResponse } from "@/lib/api"

interface ApprovalDialogProps {
  open: boolean
  ranking: RankingResponse
  agent: {
    id: string
    name: string
    specialization: string
    llm?: string
  }
  onApprove: () => void
  onReject: () => void
}

export function ApprovalDialog({
  open,
  ranking,
  agent,
  onApprove,
  onReject,
}: ApprovalDialogProps) {
  // Determine rejection reason
  const getRejectionReason = () => {
    if (!ranking.stf_compliant) {
      return "STF Compliance Violation"
    }
    if (ranking.score < 0.9) {
      return "Low Confidence Score"
    }
    return "Requires Manual Review"
  }

  // Get score color
  const getScoreColor = (score: number) => {
    if (score >= 0.9) return "text-green-500"
    if (score >= 0.7) return "text-yellow-500"
    return "text-red-500"
  }

  // Get score badge variant
  const getScoreBadgeVariant = (score: number): "default" | "secondary" | "destructive" => {
    if (score >= 0.9) return "default"
    if (score >= 0.7) return "secondary"
    return "destructive"
  }

  return (
    <AlertDialog open={open}>
      <AlertDialogContent className="max-w-2xl bg-slate-900 border-slate-700">
        <AlertDialogHeader>
          <div className="flex items-center gap-3">
            <div className="p-2 bg-yellow-500/10 rounded-lg">
              <AlertTriangle className="w-6 h-6 text-yellow-500" />
            </div>
            <div>
              <AlertDialogTitle className="text-xl text-white">
                Human Review Required
              </AlertDialogTitle>
              <AlertDialogDescription className="text-slate-400">
                {getRejectionReason()}
              </AlertDialogDescription>
            </div>
          </div>
        </AlertDialogHeader>

        <div className="space-y-4 my-4">
          {/* Agent Information */}
          <Card className="bg-slate-800/50 border-slate-700">
            <CardContent className="p-4">
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <p className="text-sm text-slate-400">Agent</p>
                  <p className="text-white font-medium">{agent.name}</p>
                  <p className="text-xs text-slate-500">{agent.specialization}</p>
                </div>
                <div>
                  <p className="text-sm text-slate-400">Model</p>
                  <p className="text-white font-medium">{agent.llm || "Unknown"}</p>
                </div>
              </div>
            </CardContent>
          </Card>

          {/* Decision Score */}
          <Card className="bg-slate-800/50 border-slate-700">
            <CardContent className="p-4">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-sm text-slate-400">Confidence Score</p>
                  <p className={`text-3xl font-bold ${getScoreColor(ranking.score)}`}>
                    {(ranking.score * 100).toFixed(1)}%
                  </p>
                  <p className="text-xs text-slate-500 mt-1">
                    Threshold: 90% for auto-approval
                  </p>
                </div>
                <Badge variant={getScoreBadgeVariant(ranking.score)} className="text-sm px-3 py-1">
                  {ranking.score >= 0.9 ? "High" : ranking.score >= 0.7 ? "Medium" : "Low"}
                </Badge>
              </div>
            </CardContent>
          </Card>

          {/* STF Compliance */}
          <Card className="bg-slate-800/50 border-slate-700">
            <CardContent className="p-4">
              <div className="flex items-center justify-between mb-2">
                <p className="text-sm text-slate-400">STF Protocol Compliance</p>
                {ranking.stf_compliant ? (
                  <div className="flex items-center gap-2 text-green-500">
                    <CheckCircle className="w-4 h-4" />
                    <span className="text-sm font-medium">Compliant</span>
                  </div>
                ) : (
                  <div className="flex items-center gap-2 text-red-500">
                    <XCircle className="w-4 h-4" />
                    <span className="text-sm font-medium">Non-Compliant</span>
                  </div>
                )}
              </div>

              {ranking.stf_violations.length > 0 && (
                <div className="mt-3 space-y-2">
                  <p className="text-xs text-slate-500 font-medium">
                    Violations Detected:
                  </p>
                  {ranking.stf_violations.map((violation, index) => (
                    <div
                      key={index}
                      className="flex items-start gap-2 p-2 bg-red-500/10 rounded border border-red-500/20"
                    >
                      <AlertCircle className="w-4 h-4 text-red-500 mt-0.5 flex-shrink-0" />
                      <p className="text-sm text-red-400">{violation}</p>
                    </div>
                  ))}
                </div>
              )}
            </CardContent>
          </Card>

          {/* Approval Status */}
          <Card className="bg-slate-800/50 border-slate-700">
            <CardContent className="p-4">
              <div className="space-y-2">
                <div className="flex items-center justify-between">
                  <p className="text-sm text-slate-400">Auto-Approved</p>
                  {ranking.approved ? (
                    <Badge variant="default" className="bg-green-500">Yes</Badge>
                  ) : (
                    <Badge variant="destructive">No</Badge>
                  )}
                </div>
                <div className="flex items-center justify-between">
                  <p className="text-sm text-slate-400">Requires Human Review</p>
                  {ranking.requires_human_review ? (
                    <Badge variant="secondary">Yes</Badge>
                  ) : (
                    <Badge variant="default">No</Badge>
                  )}
                </div>
              </div>
            </CardContent>
          </Card>

          {/* Warning Message */}
          <div className="p-4 bg-yellow-500/10 border border-yellow-500/20 rounded-lg">
            <div className="flex gap-3">
              <AlertTriangle className="w-5 h-5 text-yellow-500 flex-shrink-0 mt-0.5" />
              <div>
                <p className="text-sm font-medium text-yellow-500">
                  Manual Approval Required
                </p>
                <p className="text-xs text-slate-400 mt-1">
                  This decision did not meet the automatic approval threshold.
                  Review the details above and decide whether to proceed or cancel
                  the workflow.
                </p>
              </div>
            </div>
          </div>
        </div>

        <AlertDialogFooter>
          <AlertDialogCancel
            onClick={onReject}
            className="bg-slate-800 border-slate-700 text-white hover:bg-slate-700"
          >
            Cancel Workflow
          </AlertDialogCancel>
          <AlertDialogAction
            onClick={onApprove}
            className="bg-green-600 hover:bg-green-700 text-white"
          >
            Approve & Continue
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  )
}
