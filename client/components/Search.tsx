'use client'

import { useRouter } from 'next/navigation'
import { Button } from './ui/button'
import { Input } from './ui/input'
import { useEffect, useState } from 'react'
import { CID } from 'multiformats'
import { detectUUIDs } from '@/utils/strings'

export function Search() {
	const router = useRouter()
	const [imageId, setImageId] = useState('')
	const [isValid, setIsValid] = useState(false)
	const [type, setType] = useState<'session' | 'image' | null>(null)

	useEffect(() => {
		if (detectUUIDs(imageId)) {
			setIsValid(true)
			setType('session')
		} else {
			try {
				CID.parse(imageId)
				setIsValid(true)
				setType('image')
			} catch (error) {
				setIsValid(false)
				setType(null)
			}
		}
	}, [imageId])

	const handleKeyDown = (event: any) => {
		if (event.key === 'Enter') {
			handleNavigate()
		}
	}

	function handleClick() {
		handleNavigate()
	}

	function handleNavigate() {
		if (!imageId || !isValid) return
		router.push(`/${type === 'session' ? 'sessions' : 'images'}/${imageId}`)
	}

	return (
		<div className="flex justify-center gap-4">
			<Input
				type="search"
				placeholder="Enter a proof session or image id ..."
				className="md:w-[120px] lg:w-[420px] lg:h-[42px]"
				onChange={(e) => setImageId(e.target.value)}
				onKeyDown={handleKeyDown}
			/>

			<Button disabled={!imageId || !isValid} onClick={handleClick} className="lg:h-[42px]">
				Search
			</Button>
		</div>
	)
}
