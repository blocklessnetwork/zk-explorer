'use client'

import React, { useEffect, useState } from 'react'
import { Skeleton } from "@/components/ui/skeleton"
import { Textarea } from './ui/textarea'

export function WatViewer({ fileUrl }: { fileUrl: string }) {
	const [libraryLoaded, setLibraryLoaded] = useState(false)
	const [code, setCode] = useState('')

	useEffect(() => {
		// Load the external library dynamically
		const script = document.createElement('script')
		script.src = 'https://webassembly.github.io/wabt/demo/libwabt.js'
		script.async = true

		script.onload = () => {
			;(window as any).WabtModule().then((wabt: any) => {
				fetch(fileUrl)
					.then((response) => {
						if (!response.ok) {
							throw new Error('Network response was not ok')
						}

						return response.blob()
					})
					.then((blob) => blob.arrayBuffer())
					.then((buffer) => {
						let module = wabt!.readWasm(buffer, { readDebugNames: true })
						const result = module.toText({ foldExprs: true, inlineExport: true })
						setCode(result)
					})
					.catch((error) => {
						console.error('Error downloading file:', error)
						return null
					})
					.finally(() => setLibraryLoaded(true))
			})
		}

		document.body.appendChild(script)

		return () => {
			// Clean up the script tag if the component is unmounted
			document.body.removeChild(script)
		}
	}, [])

	if (!libraryLoaded) {
		return <div className='w-full'>
			<Skeleton className="w-1/3 h-[20px] mb-4 rounded-full" />
			<Skeleton className="w-2/3 h-[20px] mb-4 rounded-full" />
			<Skeleton className="w-2/3 h-[20px] mb-4 rounded-full" />
			<Skeleton className="w-2/3 h-[20px] mb-4 rounded-full" />
			<Skeleton className="w-2/3 h-[20px] mb-4 rounded-full" />
			<Skeleton className="w-2/3 h-[20px] mb-4 rounded-full" />
			<Skeleton className="w-5/6 h-[20px] mb-4 rounded-full" />
			<Skeleton className="w-5/6 h-[20px] mb-4 rounded-full" />
			<Skeleton className="w-5/6 h-[20px] mb-4 rounded-full" />
		</div>
	}

	// Now you can use the library in your component
	// Make sure to check if any global variables provided by the library are available before using them

	return <Textarea value={code} onChange={() => {}} />
}

export default WatViewer
