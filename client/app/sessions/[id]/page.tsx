import { Button } from '@/components/ui/button'
import { Separator } from '@/components/ui/separator'
import {
	Table,
	TableHeader,
	TableRow,
	TableHead,
	TableBody,
	TableCell
} from '@/components/ui/table'
import { Textarea } from '@/components/ui/textarea'
import { shortenString } from '@/utils/strings'
import dayjs from 'dayjs'
import { Download, Loader2 } from 'lucide-react'
import Link from 'next/link'
import { notFound } from 'next/navigation'

interface SessionRecord {
	status: string
	arguments: { value: string; arg_type: string }[]

	image_cid: string
	receipt_cid: string
	receipt_metadata: any

	method: string
	argument_type: string[]
	result_type: string

	created_at: string
	completed_at: string
}

async function getSessionDetail(id: string): Promise<SessionRecord | null> {
	try {
		const res = await fetch(`http://localhost:3005/api/proofs/${id}`, { cache: 'no-cache' })
		return res.ok ? await res.json() : null
	} catch (error) {
		return null
	}
}

export default async function SessionDetail({ params }: { params: { id: string } }) {
	const session = await getSessionDetail(params.id)
	if (!session) notFound()
	console.log('session', session)

	return (
		<div className="h-full flex-1 flex-col md:flex">
			<div className="container flex flex-1 flex-col items-start space-y-2 py-4 md:h-16">
				<div className="mb-4">
					<h2 className="text-2xl mb-1">Proof Session</h2>
					<p className="opacity-75">{params.id}</p>
				</div>
				<div className="w-full flex gap-8 mb-8">
					<div className="flex flex-1 flex-col gap-4">
						<div className="flex flex-col gap-2 w-full">
							<strong className="text-md">Arguments</strong>
							<Table>
								<TableHeader>
									<TableRow>
										<TableHead className="w w-36">Index</TableHead>
										<TableHead>Type</TableHead>
										<TableHead>Value</TableHead>
									</TableRow>
								</TableHeader>
								<TableBody>
									{session.arguments.map((a, i: number) => (
										<TableRow>
											<TableCell>{i}</TableCell>
											<TableCell>{a.arg_type}</TableCell>
											<TableCell>{a.value}</TableCell>
										</TableRow>
									))}
								</TableBody>
							</Table>
						</div>
						{session.status !== 'Completed' && (
							<div className="flex gap-2 items-center">
								<Loader2 className="mr-2 h-4 w-4 animate-spin" />
								<span>Loading ...</span>
							</div>
						)}
						{session.status === 'Completed' && (
							<>
								<div className="flex flex-col gap-2 w-full">
									<strong className="text-md">Receipt Metadata</strong>
									<Textarea
										style={{ minHeight: 250 }}
										value={JSON.stringify(session.receipt_metadata)}
									/>
								</div>
								<div className="flex">
									<Link href={`https://${session.receipt_cid}.ipfs.w3s.link`} target="_blank">
										<Button>
											<Download className="mr-2" />
											Download Receipt
										</Button>
									</Link>
								</div>
							</>
						)}
					</div>

					<div className="flex flex-col gap-4 w-1/4">
						<div className="flex flex-col gap-1">
							<strong className="text-md">Status</strong>
							<span>{session.status}</span>
						</div>
						<div className="flex flex-col gap-1">
							<strong className="text-md">Start Time</strong>
							<span>{dayjs(session.created_at).format('MMM D, YYYY h:mm:ss A')}</span>
						</div>
						<div className="flex flex-col gap-1">
							<strong className="text-md">Finish Time</strong>
							<span>
								{session.completed_at
									? dayjs(session.completed_at).format('MMM D, YYYY h:mm:ss A')
									: 'N/A'}
							</span>
						</div>
						<Separator />
						<div className="flex flex-col gap-1">
							<strong className="text-md">Image ID</strong>
							<div className="flex gap-2 items-center">
								<Link href={`/images/${session.image_cid}`}>
									{shortenString(session.image_cid)}
								</Link>
							</div>
						</div>
						<div className="flex flex-col gap-1">
							<strong className="text-md">Method</strong>
							<span>{session.method || 'zkmain'}()</span>
						</div>
						<div className="flex flex-col gap-1">
							<strong className="text-md">Argument Type</strong>
							<span>{session.argument_type ? session.argument_type.join(', ') : 'N/A'}</span>
						</div>
						<div className="flex flex-col gap-1">
							<strong className="text-md">Result Type</strong>
							<span>{session.result_type}</span>
						</div>
					</div>
				</div>
			</div>
		</div>
	)
}
