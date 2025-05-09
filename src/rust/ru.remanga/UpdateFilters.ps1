# this script is used to update filters.json
# i think it will require some fixes after some time...

function ParseString {
    param (
        [String]$s,
        [String]$begin,
        [String]$end
    )

    $idx_a = $s.IndexOf($begin)
    if ($idx_a -lt 0) {
        Write-Error "Failed to find substring beginning '$begin'"
        return $null
    }

    $idx_a = [int]$idx_a + [int]$begin.Length
    $idx_b = $s.IndexOf($end, $idx_a)
    if ($idx_b -lt $idx_a) {
        Write-Error "Failed to find substring ending '$end'"
        return $null
    }

    $len = [int]$idx_b - [int]$idx_a
    return $s.Substring($idx_a, $len)
}

function BuildFilterObject {
    param (
        [string]$type,
        [string]$name,
        [string]$id,
        [string]$id_type = $null,
        [bool]$can_exclude = $false
    )

    if ($null -ne $id_type) {
        $id = "$id_type|$id"
    }

    $filter = [PSCustomObject]@{
        type = $type
        name = $name
        id = $id
    }
    if ($can_exclude) {
        Add-Member -InputObject $filter -NotePropertyName "canExclude" -NotePropertyValue $true
    }
    return $filter
}

function GetFilterCheckItems {
    param (
        [array]$list,
        [string]$type,
        [string]$id_type = $null,
        [bool]$can_exclude = $false,
        [bool]$is_tag = $false
    )
    
    $result = @()
    foreach ($item in $list) {
        $id = $item.id
        $name = $item.name

        if ($is_tag) {
            $name = "#$name"
        }

        $filter = BuildFilterObject -type $type -name $name -id $id -id_type $id_type -can_exclude $can_exclude
        $result += $filter
    }

    return $result
}

function GetFilterNamesArray {
    param (
        [array]$list
    )
    
    $result = @()
    foreach ($item in $list) {
        $result += $item.name
    }

    return $result
}

$page = Invoke-WebRequest -Uri 'https://remanga.org/manga'
$content = $page.Content

$windowName = ParseString -s "$content" -begin "window[`"" -end "`"] = window[`""
if ($null -eq $windowName) {
    return
}

Write-Host "Found window name `"$windowName`""

$data = ParseString -s "$content" -begin "window[`"$windowName`"].push(" -end ");"
if ($null -eq $data) {
    return
}

$json = $data | ConvertFrom-Json
$json = $json.queries[0].state.data.json

$genres = GetFilterCheckItems -list $json.content.genres -type "genre" -id_type "0" -can_exclude $true
$categories = GetFilterCheckItems -list $json.content.categories -type "genre" -id_type "1" -can_exclude $true
# dunno if types can have canExclude option (didn't found something like in other sorces)
$types = GetFilterCheckItems -list $json.content.types -type "genre" -id_type "2" -can_exclude $true
$status = GetFilterCheckItems -list $json.content.status -type "check" -id_type "0"
# Translate status isn't working currently in teir API and responds with an empty array
# $translate_status = GetFilterCheckItems -list $json.content.translate_status -type "check" -id_type "1"
$age_limit = GetFilterNamesArray -list $json.content.age_limit

# hidden categories can be added here
$hidden_categories = @(
    BuildFilterObject -type "genre" -name "Хентай" -id "12" -id_type "1" -can_exclude $true
)

$out = @(
    [PSCustomObject]@{
        type = "title"
    }, [PSCustomObject]@{
        type = "group"
        name = "Тип"
        filters = $types
    }, [PSCustomObject]@{
        type = "select"
        name = "Возрастное ограничение"
        options = @("-") + $age_limit
    }, [PSCustomObject]@{
        type = "group"
        name = "Жанры"
        filters = $genres
    }, [PSCustomObject]@{
        type = "group"
        name = "Категории"
        filters = $categories + $hidden_categories
    }, [PSCustomObject]@{
        type = "group"
        name = "Статус"
        filters = $status
#    }, [PSCustomObject]@{
#        type = "group"
#        name = "Статус перевода"
#        filters = $translate_status
    }, [PSCustomObject]@{
        type = "sort"
        name = "Упорядочить"
        canAscend = $true
        options = @(
            "По новизне",
            "По количеству эпизодов",
            "По дате последнего обновления",
            "По популярности",
            "По оценке",
            "По количеству оценок",
            "По лайкам",
            "По просмотрам"
        )
        default = [PSCustomObject]@{
            index = 3
            ascending = $false
        }
    })

$out_path = Join-Path -Path (Join-Path -Path $PSScriptRoot -ChildPath "res") -ChildPath "filters.json"
ConvertTo-Json -InputObject $out -Depth 10 | Out-File -FilePath $out_path -Encoding UTF8

Write-Host $out[7]
