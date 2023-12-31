import { CID } from 'multiformats'
import { ProofRecord, columns } from './columns'
import { DataTable } from './data-table'
import { notFound } from 'next/navigation'
import { shortenString } from '@/utils/strings'
import { Separator } from '@/components/ui/separator'
import { Button } from '@/components/ui/button'
import WatViewer from '@/components/WatViewer'
import CopyClipboardButton from '@/components/CopyClipboardButton'
import { Download } from 'lucide-react'
import Link from 'next/link'
import GenerateProofButton from '@/components/GenerateProofButton'
import { API_IPFS, API_IPFS_GATEWAY, API_SERVER } from '@/constants/api'

interface ManifestRecord {
	wasm_path: string
	elf_path: string
	elf_id: string
	method: string
	argument_type: string[]
	result_type: string
}

async function getManifestDetail(id: string): Promise<ManifestRecord | null> {
	try {
		const res = await fetch(API_IPFS(`cat/${id}`))
		return res.ok ? await res.json() : null
	} catch (error) {
		return null
	}
}

async function getImageDetail(
	imageId: string
): Promise<{ manifest: ManifestRecord; files: { Name: string; Hash: string }[] } | null> {
	try {
		const res = await fetch(API_IPFS(`ls/${imageId}`))
		const data = res.ok ? await res.json() : null

		if (data.Objects && data.Objects[0] && data.Objects[0]?.Links.length === 3) {
			const files = data.Objects[0]?.Links.filter((l: any) => l.Name !== 'manifest.json')
			const manifestData = data.Objects[0]?.Links.find((l: any) => l.Name === 'manifest.json')
			const manifest = await getManifestDetail(manifestData.Hash)
			if (!manifest) return null

			return { manifest, files }
		} else {
			return null
		}
	} catch (error) {
		return null
	}
}

async function getProofs(imageId: string): Promise<ProofRecord[]> {
	try {
		const res = await fetch(API_SERVER(`proofs/by-image/${imageId}`), {
			cache: 'no-cache'
		})
		return res.ok ? await res.json() : []
	} catch (error) {
		return []
	}
}

export default async function ImageDetail({ params }: { params: { id: string } }) {
	try {
		CID.parse(params.id)
	} catch (error) {
		notFound()
	}

	const image = await getImageDetail(params.id)
	if (!image) notFound()

	const proofs = await getProofs(params.id)
	const wasmFile = image.files.find((f) => f.Name === image.manifest.wasm_path)

	return (
		<>
			<div className="h-full flex-1 flex-col md:flex">
				<div className="container flex flex-1 flex-col items-start space-y-2 py-4 md:h-16">
					<div className="mb-4">
						<h2 className="text-2xl mb-1">Image</h2>
						<p className="opacity-75">{params.id}</p>
					</div>
					<div className="w-full flex gap-8 mb-8">
						<div className="flex flex-1">
							{wasmFile && <WatViewer fileUrl={API_IPFS_GATEWAY(wasmFile.Hash)} />}
						</div>
						<div className="flex flex-col gap-4 w-1/4">
							<div className="flex flex-col gap-1">
								<strong className="text-md">Mode</strong>
								<span>{image.manifest.wasm_path ? `WASM + Wasmi Interpreter` : 'ELF Only'}</span>
							</div>
							<div className="flex flex-col gap-1">
								<strong className="text-md">Image ID</strong>
								<div className="flex gap-2 items-center">
									<span>{shortenString(image.manifest.elf_id)}</span>
									<CopyClipboardButton text={image.manifest.elf_id} />
								</div>
							</div>
							<div className="flex flex-col gap-1">
								<strong className="text-md">Method</strong>
								<span>{image.manifest.method || 'zkmain'}()</span>
							</div>
							<div className="flex flex-col gap-1">
								<strong className="text-md">Argument Type</strong>
								<span>{image.manifest.argument_type.join(', ')}</span>
							</div>
							<div className="flex flex-col gap-1">
								<strong className="text-md">Result Type</strong>
								<span>{image.manifest.result_type}</span>
							</div>
							<Separator />
							<div className="flex gap-4">
								<GenerateProofButton
									imageId={params.id}
									argumentType={image.manifest.argument_type}
								/>
								<Link href={API_IPFS_GATEWAY(params.id)} target="_blank" rel="nofollow">
									<Button variant="secondary" className="h-full">
										<Download />
									</Button>
								</Link>
							</div>
						</div>
					</div>
					<div className="w-full">
						<h2 className="text-xl mb-2 mt-4">Proof Sessions</h2>
						<DataTable columns={columns} data={proofs} />
					</div>
				</div>
			</div>
		</>
	)
}
