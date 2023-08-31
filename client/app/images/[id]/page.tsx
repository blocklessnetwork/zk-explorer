import { CID } from 'multiformats'
import { ProofRecord, columns } from './columns'
import { DataTable } from './data-table'
import { notFound } from 'next/navigation'
import { Textarea } from '@/components/ui/textarea'
import { shortenString } from '@/utils/strings'
import { Separator } from '@/components/ui/separator'
import { Button } from '@/components/ui/button'
import WatViewer from '@/components/WatViewer'

interface ManifestRecord {
	wasm_path: string
	elf_path: string
	elf_id: string
	method: string
	argument_type: string[]
	result_type: string
}

async function getManifestDetail(id: string): Promise<any> {
	const res = await fetch(`https://dweb.link/api/v0/cat/${id}`)
	return await res.json()
}

async function getImageDetail(
	imageId: string
): Promise<{ manifest: ManifestRecord; files: { Name: string; Hash: string }[] } | null> {
	const res = await fetch(`https://dweb.link/api/v0/ls/${imageId}`)
	const data = await res.json()

	if (data.Objects && data.Objects[0] && data.Objects[0]?.Links.length === 3) {
		const files = data.Objects[0]?.Links.filter((l: any) => l.Name !== 'manifest.json')
		const manifestData = data.Objects[0]?.Links.find((l: any) => l.Name === 'manifest.json')
		const manifest = await getManifestDetail(manifestData.Hash)
		return { manifest, files }
	} else {
		return null
	}
}

async function getProofs(imageId: string): Promise<ProofRecord[]> {
	const res = await fetch(`http://localhost:3005/api/proofs/by-image/${imageId}`)
	return await res.json()
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

	return (
		<>
			<div className="h-full flex-1 flex-col md:flex">
				<div className="container flex flex-1 flex-col items-start space-y-2 py-4 md:h-16">
					<div className="w-full flex gap-8 mb-8">
						<div className="flex flex-1">
							<WatViewer
								fileUrl={`https://bafybeigvj6di42f5p37jlt2tn36ug4p4sm2hq24wbjwvolnm7uhtyelni4.ipfs.w3s.link`}
							/>
						</div>
						<div className="flex flex-col gap-4 w-1/4">
							<div className="flex flex-col gap-1">
								<strong className="text-md">Mode</strong>
								<span>{image.manifest.wasm_path ? `WASM + Wasmi Interpreter` : 'ELF Only'}</span>
							</div>
							<div className="flex flex-col gap-1">
								<strong className="text-md">Image ID</strong>
								<span>{shortenString(image.manifest.elf_id)}</span>
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
							<div>
								<Button>Generate Proof</Button>
							</div>
						</div>
					</div>
					<div className="w-full">
						<h2 className="text-xl mb-2">Proof Sessions</h2>
						<DataTable columns={columns} data={proofs} />
					</div>
				</div>
			</div>
		</>
	)
}
